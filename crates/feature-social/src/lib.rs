mod post;
mod source;

use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use bot_core::serenity::{
    self,
    builder::EditMessage,
    http::{LightMethod, Request as DiscordRequest, Route},
};
use bot_core::serenity::{ChannelId, MessageId};
use bot_core::{BotState, Error};
use database::UserSocialEmbedsRepository;
use reqwest::{Client, RequestBuilder, header::USER_AGENT};
use serde_json::{Value, json};
use source::{Request as EmbedRequest, Source};

pub use source::SocialPlatform;
pub use source::twitter::{normalize_translation_language, primary_translation_language};

const MESSAGE_COMPONENT_LIMIT: usize = 40;
const EMBED_API_RETRIES: u32 = 3;
const EMBED_API_RETRY_DELAY: Duration = Duration::from_secs(1);
const EMBEDDED_MESSAGE_CACHE_LIMIT: usize = 100;
const EMBEDDED_MESSAGE_CACHE_TTL: Duration = Duration::from_secs(5 * 60);
const APP_USER_AGENT: &str = concat!("yin/", env!("CARGO_PKG_VERSION"));
const DISCORD_USER_AGENT: &str = "Discordbot/2.0";
static EMBEDDED_MESSAGES: LazyLock<Mutex<VecDeque<EmbeddedMessage>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));
static HTTP: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("valid social embed HTTP client")
});

pub async fn handle_message(
    data: &BotState,
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

    let response_ids = match send_embeds(data, ctx, message).await {
        Ok(response_ids) if response_ids.is_empty() => return Ok(()),
        Ok(response_ids) => response_ids,
        Err(error) => {
            let _ = message.react(ctx, '❌').await;
            return Err(error);
        }
    };

    remember_message(message.id, response_ids, &message.content, Instant::now());
    message
        .channel_id
        .edit_message(ctx, message.id, EditMessage::new().suppress_embeds(true))
        .await?;
    Ok(())
}

pub async fn handle_message_delete(
    ctx: &serenity::Context,
    channel_id: ChannelId,
    message_id: MessageId,
) -> Result<(), Error> {
    for response_id in take_embedded_responses(message_id) {
        channel_id.delete_message(ctx, response_id).await?;
    }
    Ok(())
}

pub async fn handle_message_update(
    data: &BotState,
    ctx: &serenity::Context,
    event: &serenity::MessageUpdateEvent,
) -> Result<(), Error> {
    let Some(content) = event.content.as_deref() else {
        return Ok(());
    };
    if !should_refresh_message(event.id, content) {
        return Ok(());
    }

    let message = event.channel_id.message(ctx, event.id).await?;
    let response_ids = match send_embeds(data, ctx, &message).await {
        Ok(response_ids) => response_ids,
        Err(error) => {
            let _ = message.react(ctx, '❌').await;
            return Err(error);
        }
    };
    let suppress_embeds = !response_ids.is_empty();

    let old_response_ids = take_embedded_responses(message.id);
    remember_message(message.id, response_ids, &message.content, Instant::now());
    for response_id in old_response_ids {
        message.channel_id.delete_message(ctx, response_id).await?;
    }
    if suppression_changed(message.flags.unwrap_or_default(), suppress_embeds) {
        message
            .channel_id
            .edit_message(
                ctx,
                message.id,
                EditMessage::new().suppress_embeds(suppress_embeds),
            )
            .await?;
    }
    Ok(())
}

struct EmbeddedMessage {
    created_at: Instant,
    message_id: MessageId,
    response_ids: Vec<MessageId>,
    content: String,
}

fn remember_message(
    message_id: MessageId,
    response_ids: Vec<MessageId>,
    content: &str,
    now: Instant,
) {
    let mut messages = EMBEDDED_MESSAGES.lock().unwrap();
    remove_expired_messages(&mut messages, now);
    if messages.len() == EMBEDDED_MESSAGE_CACHE_LIMIT {
        messages.pop_front();
    }
    messages.push_back(EmbeddedMessage {
        created_at: now,
        message_id,
        response_ids,
        content: content.to_owned(),
    });
}

fn should_refresh_message(message_id: MessageId, content: &str) -> bool {
    let mut messages = EMBEDDED_MESSAGES.lock().unwrap();
    remove_expired_messages(&mut messages, Instant::now());
    messages
        .iter()
        .find(|message| message.message_id == message_id)
        .is_some_and(|message| message.content != content)
}

fn take_embedded_responses(message_id: MessageId) -> Vec<MessageId> {
    let mut messages = EMBEDDED_MESSAGES.lock().unwrap();
    remove_expired_messages(&mut messages, Instant::now());
    let Some(index) = messages
        .iter()
        .position(|message| message.message_id == message_id)
    else {
        return Vec::new();
    };
    messages.remove(index).unwrap().response_ids
}

fn remove_expired_messages(messages: &mut VecDeque<EmbeddedMessage>, now: Instant) {
    while messages.front().is_some_and(|message| {
        now.saturating_duration_since(message.created_at) >= EMBEDDED_MESSAGE_CACHE_TTL
    }) {
        messages.pop_front();
    }
}

fn suppression_changed(flags: serenity::MessageFlags, suppress: bool) -> bool {
    flags.contains(serenity::MessageFlags::SUPPRESS_EMBEDS) != suppress
}

async fn send_embeds(
    data: &BotState,
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<Vec<MessageId>, Error> {
    let mut links = source::parse_links(&message.content);
    if links.is_empty() {
        return Ok(Vec::new());
    }
    let disabled_platforms = UserSocialEmbedsRepository::new(&data.database)
        .disabled_platforms(message.author.id.get())
        .await?;
    source::retain_enabled_links(&mut links, &disabled_platforms);
    if links.is_empty() {
        return Ok(Vec::new());
    }
    let twitter = if links
        .iter()
        .any(|link| matches!(link.source, Source::Twitter(_)))
    {
        source::twitter::options(data, ctx, message).await?
    } else {
        source::twitter::Options::default()
    };

    let mut components = Vec::new();
    for link in links {
        let source = link.source;
        let Some(request) = link.request(&twitter) else {
            continue;
        };
        let component = match request {
            EmbedRequest::FxEmbed { url, spoiler } => {
                post::component(&fetch_post(&url, post::Format::FxEmbed).await?, spoiler)
            }
            EmbedRequest::AbEmbed { url, spoiler } => {
                post::component(&fetch_post(&url, post::Format::AbEmbed).await?, spoiler)
            }
            EmbedRequest::Component { url, spoiler } => {
                let mut component = fetch_component(&url, spoiler).await?;
                if matches!(source, Source::Twitter(_)) {
                    rewrite_abembed_gifs(&mut component);
                }
                component
            }
        };
        components.push(component);
    }

    let mut response_ids = Vec::new();
    for components in component_batches(components)? {
        response_ids.push(send_payload(ctx, message, create_payload(components)).await?);
    }
    Ok(response_ids)
}

async fn fetch_component(embed_url: &str, spoiler: bool) -> Result<Value, Error> {
    let html = component_request(embed_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let mut component = component_from_html(&html)
        .ok_or_else(|| std::io::Error::other("embed response had no Discord component"))?;
    component["spoiler"] = json!(spoiler);
    Ok(component)
}

async fn send_payload(
    ctx: &serenity::Context,
    message: &serenity::Message,
    mut payload: Value,
) -> Result<MessageId, Error> {
    payload["message_reference"] = json!({"message_id": message.id});
    payload["allowed_mentions"]["replied_user"] = json!(false);
    let body = serde_json::to_vec(&payload)?;
    let response = ctx
        .http
        .fire::<serenity::Message>(
            DiscordRequest::new(
                Route::ChannelMessages {
                    channel_id: message.channel_id,
                },
                LightMethod::Post,
            )
            .body(Some(body)),
        )
        .await?;
    Ok(response.id)
}

async fn fetch_post(api_url: &str, format: post::Format) -> Result<post::Post, reqwest::Error> {
    for retry in 0..=EMBED_API_RETRIES {
        let result = async {
            let response = request(api_url).send().await?.error_for_status()?;
            post::decode(response, format).await
        }
        .await;

        match result {
            Ok(post) => return Ok(post),
            Err(_) if retry < EMBED_API_RETRIES => {
                tokio::time::sleep(EMBED_API_RETRY_DELAY * 2u32.pow(retry)).await;
            }
            Err(error) => return Err(error),
        }
    }

    unreachable!()
}

fn request(url: &str) -> RequestBuilder {
    HTTP.get(url).header(USER_AGENT, APP_USER_AGENT)
}

fn component_request(url: &str) -> RequestBuilder {
    HTTP.get(url).header(USER_AGENT, DISCORD_USER_AGENT)
}

fn component_from_html(html: &str) -> Option<Value> {
    let json = html
        .split_once(r#"<script id="discord:component-embed" type="application/json">"#)?
        .1
        .split_once("</script>")?
        .0;
    serde_json::from_str::<Value>(json)
        .ok()?
        .get("component")
        .cloned()
}

fn rewrite_abembed_gifs(value: &mut Value) {
    match value {
        Value::String(url)
            if url.starts_with("https://gif.abembed.com/") && url.ends_with(".gif") =>
        {
            url.truncate(url.len() - ".gif".len());
            url.push_str(".webp");
        }
        Value::Array(values) => values.iter_mut().for_each(rewrite_abembed_gifs),
        Value::Object(values) => values.values_mut().for_each(rewrite_abembed_gifs),
        _ => {}
    }
}

fn create_payload(components: Vec<Value>) -> Value {
    json!({
        "flags": 1 << 15,
        "allowed_mentions": {"parse": []},
        "components": components,
    })
}

fn component_batches(components: Vec<Value>) -> Result<Vec<Vec<Value>>, std::io::Error> {
    let mut batches = Vec::new();
    let mut batch = Vec::new();
    let mut batch_size = 0;

    for component in components {
        let size = component_count(&component);
        if size > MESSAGE_COMPONENT_LIMIT {
            return Err(std::io::Error::other(format!(
                "social embed has {size} components; Discord allows {MESSAGE_COMPONENT_LIMIT}"
            )));
        }
        if batch_size + size > MESSAGE_COMPONENT_LIMIT {
            batches.push(std::mem::take(&mut batch));
            batch_size = 0;
        }
        batch_size += size;
        batch.push(component);
    }

    if !batch.is_empty() {
        batches.push(batch);
    }
    Ok(batches)
}

fn component_count(component: &Value) -> usize {
    1 + component
        .get("components")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(component_count)
        .sum::<usize>()
        + component.get("accessory").map_or(0, component_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_multiple_supported_links_in_message_order() {
        let twitter = source::twitter::Options::default();
        let requests = source::parse_links(concat!(
            "first https://x.com/jack/status/20 ",
            "then ||https://open.spotify.com/track/11dFghVXANMlKmJXsNCbNl|| ",
            "and https://vm.tiktok.com/ZN88Qw7ns/"
        ))
        .into_iter()
        .filter_map(|link| link.request(&twitter))
        .collect::<Vec<_>>();

        assert_eq!(
            requests,
            vec![
                EmbedRequest::FxEmbed {
                    url: "https://api.fxtwitter.com/2/status/20?lang=en".to_owned(),
                    spoiler: false,
                },
                EmbedRequest::Component {
                    url: "https://open.spqtify.com/track/11dFghVXANMlKmJXsNCbNl".to_owned(),
                    spoiler: true,
                },
                EmbedRequest::AbEmbed {
                    url: "https://abembed.com/api/tiktok/ZN88Qw7ns".to_owned(),
                    spoiler: false,
                },
            ]
        );
    }

    #[test]
    fn extracts_discord_components_from_html() {
        let component = component_from_html(concat!(
            r#"<html><script id="discord:component-embed" type="application/json">"#,
            r##"{"component":{"type":17,"accent_color":8505551,"components":[{"type":10,"content":"text"}]}}"##,
            "</script></html>"
        ))
        .unwrap();
        let payload = create_payload(vec![component]);

        assert!(payload.get("content").is_none());
        assert_eq!(payload["flags"], 1 << 15);
        assert_eq!(payload["components"][0]["components"][0]["content"], "text");
        assert_eq!(payload["components"][0]["accent_color"], 8505551);
        assert!(component_from_html("<html></html>").is_none());
    }

    #[test]
    fn rewrites_only_abembed_gif_urls_to_webp() {
        let mut component = json!({
            "components": [{"media": {"url": "https://gif.abembed.com/tweet_video/a.gif"}}],
            "other": "https://example.com/a.gif",
        });

        rewrite_abembed_gifs(&mut component);

        assert_eq!(
            component["components"][0]["media"]["url"],
            "https://gif.abembed.com/tweet_video/a.webp"
        );
        assert_eq!(component["other"], "https://example.com/a.gif");
    }

    #[test]
    fn batches_embeds_at_discords_component_limit() {
        let component = |children| {
            json!({
                "type": 17,
                "components": (0..children)
                    .map(|_| json!({"type": 10, "content": "text"}))
                    .collect::<Vec<_>>(),
            })
        };
        let batches = component_batches(vec![component(19), component(19), component(1)]).unwrap();

        assert_eq!(batches.iter().map(Vec::len).collect::<Vec<_>>(), [2, 1]);
        assert_eq!(batches[0].iter().map(component_count).sum::<usize>(), 40);
        assert_eq!(batches[1].iter().map(component_count).sum::<usize>(), 2);
    }

    #[test]
    fn only_updates_suppression_when_its_state_changes() {
        assert!(!suppression_changed(
            serenity::MessageFlags::SUPPRESS_EMBEDS,
            true,
        ));
        assert!(suppression_changed(serenity::MessageFlags::empty(), true));
        assert!(suppression_changed(
            serenity::MessageFlags::SUPPRESS_EMBEDS,
            false,
        ));
    }

    #[test]
    fn identifies_the_bot_to_embed_apis() {
        let request = request("https://api.fxtwitter.com/2/status/20")
            .build()
            .unwrap();

        assert_eq!(request.headers()[USER_AGENT], APP_USER_AGENT);
    }

    #[test]
    fn embed_api_retries_use_increasing_backoff() {
        assert_eq!(
            (0..EMBED_API_RETRIES)
                .map(|retry| EMBED_API_RETRY_DELAY * 2u32.pow(retry))
                .collect::<Vec<_>>(),
            [
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(4)
            ]
        );
    }

    #[test]
    fn embedded_message_cache_maps_responses_expires_entries_and_stays_bounded() {
        let now = Instant::now();
        let mut messages = EMBEDDED_MESSAGES.lock().unwrap();
        messages.clear();
        messages.push_back(EmbeddedMessage {
            created_at: now - EMBEDDED_MESSAGE_CACHE_TTL,
            message_id: serenity::MessageId::new(1),
            response_ids: vec![serenity::MessageId::new(101)],
            content: "expired".to_owned(),
        });
        drop(messages);

        for id in 2..=EMBEDDED_MESSAGE_CACHE_LIMIT as u64 + 2 {
            remember_message(
                serenity::MessageId::new(id),
                vec![serenity::MessageId::new(id + 100)],
                &format!("content {id}"),
                now,
            );
        }

        let messages = EMBEDDED_MESSAGES.lock().unwrap();
        assert_eq!(messages.len(), EMBEDDED_MESSAGE_CACHE_LIMIT);
        assert_eq!(
            messages.front().unwrap().message_id,
            serenity::MessageId::new(3)
        );
        assert_eq!(
            messages.back().unwrap().message_id,
            serenity::MessageId::new(102)
        );
        drop(messages);

        assert!(!should_refresh_message(
            serenity::MessageId::new(3),
            "content 3"
        ));
        assert!(should_refresh_message(
            serenity::MessageId::new(3),
            "edited content"
        ));
        assert_eq!(
            take_embedded_responses(serenity::MessageId::new(3)),
            vec![serenity::MessageId::new(103)]
        );
        assert!(take_embedded_responses(serenity::MessageId::new(3)).is_empty());
        assert_eq!(
            take_embedded_responses(serenity::MessageId::new(102)),
            vec![serenity::MessageId::new(202)]
        );

        remember_message(
            serenity::MessageId::new(200),
            vec![serenity::MessageId::new(300), serenity::MessageId::new(301)],
            "two embeds",
            now,
        );
        assert_eq!(
            take_embedded_responses(serenity::MessageId::new(200)),
            [serenity::MessageId::new(300), serenity::MessageId::new(301)]
        );

        EMBEDDED_MESSAGES.lock().unwrap().clear();
    }
}

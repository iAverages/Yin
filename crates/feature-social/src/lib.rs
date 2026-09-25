use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use bot_core::serenity::{
    self,
    builder::EditMessage,
    http::{LightMethod, Request, Route},
};
use bot_core::serenity::{ChannelId, MessageId};
use bot_core::{BotState, Error};
use reqwest::{Client, RequestBuilder, Url, header::USER_AGENT};
use serde::Deserialize;
use serde_json::{Value, json};

const TWITTER_API: &str = "https://api.fxtwitter.com/2/status/";
const BLUESKY_API: &str = "https://api.fxbsky.app/2/status/";
const ABEMBED_API: &str = "https://i.kirsi.dev/api/";
const TWITTER_HOSTS: &[&str] = &[
    "x.com",
    "www.x.com",
    "twitter.com",
    "www.twitter.com",
    "mobile.twitter.com",
];
const BLUESKY_HOSTS: &[&str] = &["bsky.app", "www.bsky.app"];
const INSTAGRAM_HOSTS: &[&str] = &["instagram.com", "www.instagram.com"];
const FACEBOOK_HOSTS: &[&str] = &["facebook.com", "www.facebook.com", "m.facebook.com"];
const TIKTOK_HOSTS: &[&str] = &["tiktok.com", "www.tiktok.com", "m.tiktok.com"];
const TIKTOK_SHORT_HOSTS: &[&str] = &["vt.tiktok.com", "vm.tiktok.com"];
const DESCRIPTION_LIMIT: usize = 500;
const MESSAGE_COMPONENT_LIMIT: usize = 40;
const MEDIA_GALLERY_ITEM_LIMIT: usize = 10;
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
        .expect("valid FxEmbed HTTP client")
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
    let mut links = embed_links(&message.content, "en");
    if links
        .iter()
        .any(|link| matches!(link, EmbedLink::Api(url, _) if url.starts_with(TWITTER_API)))
    {
        let language = guild_translation_language(data, ctx, message).await?;
        links = embed_links(&message.content, &language);
    }

    let mut components = Vec::new();
    for link in links {
        let component = match link {
            EmbedLink::Api(url, spoiler) => {
                create_post_component(&fetch_post(&url).await?, spoiler)
            }
            EmbedLink::Spotify(url, spoiler) => fetch_spotify_component(&url, spoiler).await?,
        };
        components.push(component);
    }

    let mut response_ids = Vec::new();
    for components in component_batches(components)? {
        response_ids.push(send_payload(ctx, message, create_payload(components)).await?);
    }
    Ok(response_ids)
}

async fn guild_translation_language(
    data: &BotState,
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<String, Error> {
    let Some(guild_id) = message.guild_id else {
        return Ok("en".to_owned());
    };
    let default = ctx
        .cache
        .guild(guild_id)
        .and_then(|guild| primary_translation_language(&guild.preferred_locale))
        .unwrap_or_else(|| "en".to_owned());
    let configured = database::GuildSettingsRepository::new(&data.database)
        .find_by_guild_id(guild_id.get())
        .await?
        .and_then(|settings| settings.translation_language)
        .and_then(|language| normalize_translation_language(&language));
    Ok(configured.unwrap_or(default))
}

async fn fetch_spotify_component(embed_url: &str, spoiler: bool) -> Result<Value, Error> {
    let html = spqtify_request(embed_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let mut component = spqtify_component(&html)
        .ok_or_else(|| std::io::Error::other("spqtify response had no Discord component"))?;
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
            Request::new(
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

async fn fetch_post(api_url: &str) -> Result<Post, reqwest::Error> {
    for retry in 0..=EMBED_API_RETRIES {
        let result = async {
            let response = request(api_url).send().await?.error_for_status()?;
            if api_url.starts_with(ABEMBED_API) {
                Ok(response.json::<LinkFixedPost>().await?.into())
            } else {
                Ok(response.json::<Response>().await?.status)
            }
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

fn spqtify_request(url: &str) -> RequestBuilder {
    HTTP.get(url).header(USER_AGENT, DISCORD_USER_AGENT)
}

fn parse_url(word: &str) -> Option<(Url, bool)> {
    let word = word.trim_matches(|c: char| "<>()[]{}\"',.!?".contains(c));
    let (word, spoiler) = match word
        .strip_prefix("||")
        .and_then(|word| word.strip_suffix("||"))
    {
        Some(word) => (word, true),
        None => (word, false),
    };
    let url = Url::parse(word.trim_matches(|c: char| "<>()[]{}\"',.!?".contains(c))).ok()?;
    Some((url, spoiler))
}

fn tiktok_short_api_url(word: &str) -> Option<(String, bool)> {
    let (url, spoiler) = parse_url(word)?;
    let parts: Vec<_> = url.path_segments()?.collect();
    let shortcode = match parts.as_slice() {
        [shortcode] | [shortcode, ""] => *shortcode,
        _ => return None,
    };
    (url.scheme() == "https"
        && TIKTOK_SHORT_HOSTS.contains(&url.host_str()?)
        && (4..=32).contains(&shortcode.len())
        && shortcode.bytes().all(|byte| byte.is_ascii_alphanumeric()))
    .then(|| (format!("{ABEMBED_API}tiktok/{shortcode}"), spoiler))
}

fn spotify_embed_url(content: &str) -> Option<(String, bool)> {
    content.split_whitespace().find_map(|word| {
        let (mut url, spoiler) = parse_url(word)?;
        let parts: Vec<_> = url.path_segments()?.collect();
        match parts.as_slice() {
            ["track" | "episode" | "album" | "playlist", id]
                if url.scheme() == "https"
                    && url.host_str() == Some("open.spotify.com")
                    && id.len() == 22
                    && id.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
            {
                url.set_host(Some("open.spqtify.com")).ok()?;
                Some((url.into(), spoiler))
            }
            _ => None,
        }
    })
}

#[derive(Debug, PartialEq)]
enum EmbedLink {
    Api(String, bool),
    Spotify(String, bool),
}

fn embed_links(content: &str, language: &str) -> Vec<EmbedLink> {
    content
        .split_whitespace()
        .filter_map(|word| {
            spotify_embed_url(word)
                .map(|(url, spoiler)| EmbedLink::Spotify(url, spoiler))
                .or_else(|| {
                    api_url(word, language).map(|(url, spoiler)| EmbedLink::Api(url, spoiler))
                })
                .or_else(|| {
                    tiktok_short_api_url(word).map(|(url, spoiler)| EmbedLink::Api(url, spoiler))
                })
        })
        .collect()
}

fn spqtify_component(html: &str) -> Option<Value> {
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

fn api_url(content: &str, language: &str) -> Option<(String, bool)> {
    content.split_whitespace().find_map(|word| {
        let (url, spoiler) = parse_url(word)?;
        let host = url.host_str()?;
        let parts: Vec<_> = url.path_segments()?.collect();

        match parts.as_slice() {
            [_, "status", id, ..]
                if TWITTER_HOSTS.contains(&host)
                    && (2..=20).contains(&id.len())
                    && id.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                let language = parts
                    .get(3)
                    .and_then(|language| normalize_translation_language(language))
                    .unwrap_or_else(|| language.to_owned());
                Some((format!("{TWITTER_API}{id}?lang={language}"), spoiler))
            }
            ["profile", handle, "post", rkey, ..]
                if BLUESKY_HOSTS.contains(&host) && !handle.is_empty() && !rkey.is_empty() =>
            {
                Some((format!("{BLUESKY_API}{handle}/{rkey}"), spoiler))
            }
            ["p" | "reel" | "reels" | "tv", shortcode, ..]
                if INSTAGRAM_HOSTS.contains(&host)
                    && !shortcode.is_empty()
                    && shortcode.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
                    }) =>
            {
                Some((format!("{ABEMBED_API}instagram/p/{shortcode}"), spoiler))
            }
            ["share", kind @ ("p" | "r" | "v"), code, ..]
                if FACEBOOK_HOSTS.contains(&host)
                    && !code.is_empty()
                    && code.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
            {
                Some((
                    format!("{ABEMBED_API}facebook/share/{kind}/{code}"),
                    spoiler,
                ))
            }
            ["share", code, ..]
                if FACEBOOK_HOSTS.contains(&host)
                    && !code.is_empty()
                    && code.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
            {
                Some((format!("{ABEMBED_API}facebook/share/{code}"), spoiler))
            }
            [username, kind @ ("video" | "photo"), id, ..]
                if TIKTOK_HOSTS.contains(&host)
                    && username.starts_with('@')
                    && (5..=30).contains(&id.len())
                    && id.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                Some((
                    format!("{ABEMBED_API}tiktok/{username}/{kind}/{id}"),
                    spoiler,
                ))
            }
            ["t", id, ..]
                if TIKTOK_HOSTS.contains(&host)
                    && !id.is_empty()
                    && id.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
            {
                Some((format!("{ABEMBED_API}tiktok/{id}"), spoiler))
            }
            _ => None,
        }
    })
}

pub fn normalize_translation_language(language: &str) -> Option<String> {
    let language = language.trim().to_ascii_lowercase().replace('_', "-");
    let language = match language.as_str() {
        "zh" | "cn" | "zh-hans" => "zh-cn",
        "tw" | "hk" | "zh-hk" | "zh-mo" | "zh-hant" => "zh-tw",
        "jp" => "ja",
        "kr" => "ko",
        "ua" => "uk",
        language => language,
    };
    let mut parts = language.split('-');
    let primary = parts.next()?;
    let subtag = parts.next();
    if parts.next().is_some()
        || !(2..=3).contains(&primary.len())
        || !primary.bytes().all(|byte| byte.is_ascii_alphabetic())
        || subtag.is_some_and(|subtag| {
            !(2..=4).contains(&subtag.len())
                || !subtag.bytes().all(|byte| byte.is_ascii_alphabetic())
        })
    {
        return None;
    }
    Some(language.to_owned())
}

pub fn primary_translation_language(locale: &str) -> Option<String> {
    normalize_translation_language(locale.split(['-', '_']).next()?)
}

fn create_post_component(post: &Post, spoiler: bool) -> Value {
    let mut components = Vec::new();
    append_post(&mut components, post, false);

    if let Some(quote) = &post.quote {
        components.push(json!({"type": 14, "divider": true, "spacing": 1}));
        match quote {
            Quote::Post(quote) => append_post(&mut components, quote, true),
            Quote::Tombstone(tombstone) => components.push(json!({
                "type": 10,
                "content": format!(
                    "**Quoted post unavailable**\n{}",
                    tombstone.message.as_deref().unwrap_or(&tombstone.reason),
                ),
            })),
        }
    }

    json!({
        "type": 17,
        "spoiler": spoiler,
        "accent_color": match post.provider.as_str() {
            "bluesky" => 0x1185fe,
            "instagram" => 0xce0071,
            "facebook" => 0x1877f2,
            _ => 0x1d9bf0,
        },
        "components": components,
    })
}

fn append_post(components: &mut Vec<Value>, post: &Post, quoted: bool) {
    let author = post.author.as_ref();
    let text = if post.text.is_empty() {
        format!("{} post", provider_name(&post.provider))
    } else {
        truncate(&post.text, DESCRIPTION_LIMIT)
    };
    let header = json!({
        "type": 10,
        "content": match author {
            Some(author) => format!(
                "{}**{}**\n{}",
                if quoted { "**Quoted post**\n" } else { "" },
                author_name(author),
                text,
            ),
            None => format!("{}{}", if quoted { "**Quoted post**\n" } else { "" }, text),
        },
    });

    if let Some(translation) = &post.translation {
        components.push(json!({
            "type": 10,
            "content": format!(
                "**Translation ({})**\n{}",
                translation.target_lang,
                truncate(&translation.text, DESCRIPTION_LIMIT),
            ),
        }));
        components.push(json!({"type": 14, "divider": true, "spacing": 1}));
    }

    components.push(match author.and_then(|author| author.avatar_url.as_ref()) {
        Some(avatar_url) => json!({
            "type": 9,
            "components": [header],
            "accessory": {"type": 11, "media": {"url": avatar_url}},
        }),
        None => header,
    });

    for media in post.media.all.chunks(MEDIA_GALLERY_ITEM_LIMIT) {
        components.push(json!({
            "type": 12,
            "items": media.iter().map(|media| json!({
                "media": {"url": media_url(post, media)},
            })).collect::<Vec<_>>(),
        }));
    }

    let stats = [
        post.likes.map(|value| format!("❤️ {value}")),
        post.reposts.map(|value| format!("🔁 {value}")),
        post.replies.map(|value| format!("💬 {value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("   ");
    let provider = provider_name(&post.provider);
    let link = format!("[View on {provider}]({})", post.url);
    components.push(json!({
        "type": 10,
        "content": if stats.is_empty() { link } else { format!("{stats}\n{link}") },
    }));
}

fn author_name(author: &Author) -> String {
    if author.screen_name.is_empty() || author.name.contains(&format!("@{}", author.screen_name)) {
        author.name.clone()
    } else {
        format!("{} (@{})", author.name, author.screen_name)
    }
}

fn provider_name(provider: &str) -> &str {
    match provider {
        "twitter" => "X / Twitter",
        "bluesky" => "Bluesky",
        "instagram" => "Instagram",
        "facebook" => "Facebook",
        "tiktok" => "TikTok",
        provider => provider,
    }
}

fn media_url(post: &Post, media: &MediaItem) -> String {
    if media.kind != "gif" {
        return media.url.clone();
    }

    if post.provider == "twitter" {
        let Ok(mut url) = Url::parse(&media.url) else {
            return media.url.clone();
        };
        if url.host_str() != Some("video.twimg.com") || !url.path().ends_with(".mp4") {
            return media.url.clone();
        }
        let path = url.path().trim_end_matches(".mp4").to_owned() + ".gif";
        if url.set_host(Some("gif.fxtwitter.com")).is_ok() {
            url.set_path(&path);
            return url.into();
        }
        return media.url.clone();
    }

    let host = match post.provider.as_str() {
        "bluesky" => "d.fxbsky.app",
        _ => return media.url.clone(),
    };
    let Ok(mut url) = Url::parse(&post.url) else {
        return media.url.clone();
    };
    if url.set_host(Some(host)).is_err() {
        return media.url.clone();
    }
    url.into()
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_owned();
    }
    value.chars().take(limit - 3).collect::<String>() + "..."
}

#[derive(Deserialize)]
struct Response {
    status: Post,
}

#[derive(Deserialize)]
struct Post {
    url: String,
    text: String,
    likes: Option<u64>,
    reposts: Option<u64>,
    replies: Option<u64>,
    author: Option<Author>,
    #[serde(default)]
    media: Media,
    provider: String,
    translation: Option<Translation>,
    quote: Option<Quote>,
}

#[derive(Deserialize)]
struct Translation {
    text: String,
    target_lang: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Quote {
    Post(Box<Post>),
    Tombstone(Tombstone),
}

#[derive(Deserialize)]
struct Tombstone {
    reason: String,
    message: Option<String>,
}

#[derive(Deserialize)]
struct Author {
    name: String,
    screen_name: String,
    avatar_url: Option<String>,
}

#[derive(Default, Deserialize)]
struct Media {
    #[serde(default)]
    all: Vec<MediaItem>,
}

#[derive(Deserialize)]
struct MediaItem {
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

#[derive(Deserialize)]
struct LinkFixedPost {
    url: String,
    description: Option<String>,
    author: Option<LinkFixedAuthor>,
    stats: LinkFixedStats,
    media: Vec<MediaItem>,
    platform: String,
}

#[derive(Deserialize)]
struct LinkFixedAuthor {
    name: String,
    username: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct LinkFixedStats {
    likes: Option<u64>,
    reposts: Option<u64>,
    comments: Option<u64>,
}

impl From<LinkFixedPost> for Post {
    fn from(post: LinkFixedPost) -> Self {
        Self {
            url: post.url,
            text: post.description.unwrap_or_default(),
            likes: post.stats.likes,
            reposts: post.stats.reposts,
            replies: post.stats.comments,
            author: post.author.map(|author| Author {
                name: author.name,
                screen_name: author.username.unwrap_or_default(),
                avatar_url: author.avatar_url,
            }),
            media: Media { all: post.media },
            provider: post.platform,
            translation: None,
            quote: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_post_links() {
        assert_eq!(
            api_url("look <https://x.com/jack/status/20?s=20>", "fr"),
            Some((
                "https://api.fxtwitter.com/2/status/20?lang=fr".to_owned(),
                false
            ))
        );
        assert_eq!(
            api_url("https://x.com/jack/status/20/en", "fr"),
            Some((
                "https://api.fxtwitter.com/2/status/20?lang=en".to_owned(),
                false
            ))
        );
        assert_eq!(
            api_url("https://x.com/jack/status/20/jp", "fr"),
            Some((
                "https://api.fxtwitter.com/2/status/20?lang=ja".to_owned(),
                false
            ))
        );
        assert_eq!(
            api_url("https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l", "fr"),
            Some((
                "https://api.fxbsky.app/2/status/bsky.app/3l6oveex3ii2l".to_owned(),
                false,
            ))
        );
        assert_eq!(
            api_url("https://www.instagram.com/reels/DbCP6xzRzdo/", "fr"),
            Some((
                "https://i.kirsi.dev/api/instagram/p/DbCP6xzRzdo".to_owned(),
                false,
            ))
        );
        assert_eq!(
            api_url(
                "https://www.tiktok.com/@kopilawak/video/7665179028352945426",
                "fr"
            ),
            Some((
                "https://i.kirsi.dev/api/tiktok/@kopilawak/video/7665179028352945426".to_owned(),
                false,
            ))
        );
        assert_eq!(
            api_url("https://www.tiktok.com/t/ZP8T6SD9F", "fr"),
            Some(("https://i.kirsi.dev/api/tiktok/ZP8T6SD9F".to_owned(), false,))
        );
        for (url, route) in [
            (
                "https://www.facebook.com/share/p/1DL7Uac7xc/?mibextid=wwXIfr",
                "share/p/1DL7Uac7xc",
            ),
            (
                "https://www.facebook.com/share/r/1MbWc8LmKs/?mibextid=wwXIfr",
                "share/r/1MbWc8LmKs",
            ),
            (
                "https://www.facebook.com/share/1DTvc3LK3k/?mibextid=wwXIfr",
                "share/1DTvc3LK3k",
            ),
            (
                "https://www.facebook.com/share/p/1KHNpv7kw6/?mibextid=wwXIfr",
                "share/p/1KHNpv7kw6",
            ),
            (
                "https://www.facebook.com/share/r/1EudwBSGoX/?mibextid=wwXIfr",
                "share/r/1EudwBSGoX",
            ),
            (
                "https://www.facebook.com/share/v/1DJnCygSRY/",
                "share/v/1DJnCygSRY",
            ),
        ] {
            assert_eq!(
                api_url(url, "fr"),
                Some((format!("https://i.kirsi.dev/api/facebook/{route}"), false,))
            );
        }
    }

    #[test]
    fn supports_fxtwitter_languages_and_aliases() {
        assert_eq!(
            normalize_translation_language("pt-BR").as_deref(),
            Some("pt-br")
        );
        assert_eq!(
            normalize_translation_language("zh-Hant").as_deref(),
            Some("zh-tw")
        );
        assert_eq!(normalize_translation_language("jp").as_deref(), Some("ja"));
        assert_eq!(
            normalize_translation_language("fil").as_deref(),
            Some("fil")
        );
        assert_eq!(normalize_translation_language("unknown"), None);
        assert_eq!(primary_translation_language("pt-BR").as_deref(), Some("pt"));
    }

    #[test]
    fn rewrites_supported_spotify_links_for_spqtify_embeds() {
        for kind in ["track", "episode", "album", "playlist"] {
            assert_eq!(
                spotify_embed_url(&format!(
                    "listen <https://open.spotify.com/{kind}/11dFghVXANMlKmJXsNCbNl?si=abc>"
                )),
                Some((
                    format!("https://open.spqtify.com/{kind}/11dFghVXANMlKmJXsNCbNl?si=abc"),
                    false,
                ))
            );
        }
        assert_eq!(
            spotify_embed_url("https://open.spotify.com/artist/0LyfQWJT6nXafLPZqxe9Of"),
            None
        );
        assert_eq!(
            spotify_embed_url("https://open.spotify.com.example/track/11dFghVXANMlKmJXsNCbNl"),
            None
        );

        let component = spqtify_component(concat!(
            r#"<html><script id="discord:component-embed" type="application/json">"#,
            r##"{"component":{"type":17,"accent_color":8505551,"components":[{"type":10,"content":"# [Cut To The Feeling](https://open.spqtify.com/track/11dFghVXANMlKmJXsNCbNl)"}]}}"##,
            "</script></html>"
        ))
        .unwrap();
        let payload = create_payload(vec![component]);
        assert_eq!(
            payload["components"][0]["components"][0]["content"],
            "# [Cut To The Feeling](https://open.spqtify.com/track/11dFghVXANMlKmJXsNCbNl)"
        );
        assert_eq!(payload["components"][0]["accent_color"], 8505551);
        assert!(spqtify_component("<html></html>").is_none());
    }

    #[test]
    fn creates_api_urls_for_tiktok_short_links() {
        assert_eq!(
            tiktok_short_api_url("<https://vt.tiktok.com/ZSVhvYhGN/>"),
            Some(("https://i.kirsi.dev/api/tiktok/ZSVhvYhGN".to_owned(), false))
        );
        assert_eq!(
            tiktok_short_api_url("https://vm.tiktok.com/ZN88Qw7ns/"),
            Some(("https://i.kirsi.dev/api/tiktok/ZN88Qw7ns".to_owned(), false))
        );
        assert_eq!(
            tiktok_short_api_url("https://vt.tiktok.com.example/ZSVhvYhGN/"),
            None
        );
    }

    #[test]
    fn finds_multiple_supported_links_in_message_order() {
        assert_eq!(
            embed_links(
                concat!(
                    "first https://x.com/jack/status/20 ",
                    "then ||https://open.spotify.com/track/11dFghVXANMlKmJXsNCbNl|| ",
                    "and https://vm.tiktok.com/ZN88Qw7ns/"
                ),
                "en",
            ),
            vec![
                EmbedLink::Api(
                    "https://api.fxtwitter.com/2/status/20?lang=en".to_owned(),
                    false,
                ),
                EmbedLink::Spotify(
                    "https://open.spqtify.com/track/11dFghVXANMlKmJXsNCbNl".to_owned(),
                    true,
                ),
                EmbedLink::Api("https://i.kirsi.dev/api/tiktok/ZN88Qw7ns".to_owned(), false,),
            ]
        );
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
    fn ignores_non_post_and_lookalike_links() {
        assert_eq!(api_url("https://x.com/jack", "en"), None);
        assert_eq!(api_url("https://x.com.example/jack/status/20", "en"), None);
        assert_eq!(api_url("https://bsky.app/profile/bsky.app", "en"), None);
        assert_eq!(api_url("https://www.instagram.com/poster/", "en"), None);
    }

    #[test]
    fn spoilers_the_generated_container_for_a_spoilered_link() {
        let (url, spoiler) = parse_url("||<https://x.com/jack/status/20>||").unwrap();
        assert_eq!(url.as_str(), "https://x.com/jack/status/20");
        assert!(spoiler);
        assert_eq!(
            api_url("||https://x.com/jack/status/20||", "en"),
            Some((
                "https://api.fxtwitter.com/2/status/20?lang=en".to_owned(),
                true
            ))
        );

        let component = create_post_component(&post_with_media(1), spoiler);
        assert_eq!(component["spoiler"], true);
    }

    #[test]
    fn decodes_shared_api_response_and_media() {
        let response: Response = serde_json::from_str(
            r#"{
                "status": {
                    "url": "https://x.com/user/status/123",
                    "text": "post text",
                    "likes": 4,
                    "reposts": 3,
                    "replies": 2,
                    "provider": "twitter",
                    "translation": {
                        "text": "translated post text",
                        "source_lang": "ja",
                        "target_lang": "en"
                    },
                    "author": {
                        "name": "User",
                        "screen_name": "user",
                        "url": "https://x.com/user",
                        "avatar_url": null
                    },
                    "media": {"all": [
                        {"type": "photo", "url": "https://example.com/image.jpg"},
                        {"type": "gif", "url": "https://video.twimg.com/tweet_video/animation.mp4", "format": "video/mp4"}
                    ]},
                    "quote": {
                        "url": "https://x.com/quoted/status/456",
                        "text": "quoted text",
                        "likes": 8,
                        "reposts": 7,
                        "replies": 6,
                        "provider": "twitter",
                        "author": {
                            "name": "Quoted User",
                            "screen_name": "quoted",
                            "avatar_url": null
                        },
                        "media": {"all": [
                            {"type": "photo", "url": "https://example.com/quoted.jpg"}
                        ]}
                    }
                }
            }"#,
        )
        .unwrap();

        assert_eq!(response.status.media.all.len(), 2);
        let message = create_payload(vec![create_post_component(&response.status, false)]);
        assert!(message.get("content").is_none());
        assert_eq!(message["flags"], 1 << 15);
        assert_eq!(
            message["components"][0]["components"][0]["content"],
            "**Translation (en)**\ntranslated post text"
        );
        assert_eq!(message["components"][0]["components"][1]["type"], 14);
        assert_eq!(
            message["components"][0]["components"][2]["content"],
            "**User (@user)**\npost text"
        );
        assert_eq!(
            message["components"][0]["components"][3]["items"][1]["media"]["url"],
            "https://gif.fxtwitter.com/tweet_video/animation.gif"
        );
        assert!(
            message["components"][0]["components"][6]["content"]
                .as_str()
                .unwrap()
                .contains("quoted text")
        );
        assert_eq!(
            message["components"][0]["components"][7]["items"][0]["media"]["url"],
            "https://example.com/quoted.jpg"
        );
    }

    #[test]
    fn truncates_on_character_boundaries() {
        assert_eq!(truncate("hello", 5), "hello");
        assert_eq!(truncate("😀abcd", 4), "😀...");
    }

    #[test]
    fn decodes_link_fixed_response() {
        let post: Post = serde_json::from_str::<LinkFixedPost>(
            r#"{
                "platform": "instagram",
                "id": "DbCP6xzRzdo",
                "url": "https://www.instagram.com/p/DbCP6xzRzdo/",
                "description": "post text",
                "author": {
                    "name": "User (@user)",
                    "username": "user",
                    "url": "https://www.instagram.com/user/",
                    "avatar_url": null
                },
                "stats": {"likes": 4, "reposts": null, "comments": 2},
                "media": [{
                    "type": "image",
                    "url": "https://example.com/image.jpg",
                    "width": 100,
                    "height": 100
                }]
            }"#,
        )
        .unwrap()
        .into();

        let message = create_payload(vec![create_post_component(&post, false)]);
        assert_eq!(message["components"][0]["accent_color"], 0xce0071);
        assert_eq!(
            message["components"][0]["components"][0]["content"],
            "**User (@user)**\npost text"
        );
        assert_eq!(
            message["components"][0]["components"][1]["items"][0]["media"]["url"],
            "https://example.com/image.jpg"
        );
        assert_eq!(
            message["components"][0]["components"][2]["content"],
            "❤️ 4   💬 2\n[View on Instagram](https://www.instagram.com/p/DbCP6xzRzdo/)"
        );
    }

    #[test]
    fn splits_more_than_ten_images_across_media_galleries() {
        let message = create_payload(vec![create_post_component(&post_with_media(11), false)]);
        let components = message["components"][0]["components"].as_array().unwrap();
        assert_eq!(components[1]["items"].as_array().unwrap().len(), 10);
        assert_eq!(components[2]["items"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn supports_discords_maximum_number_of_images() {
        let gallery_count = MESSAGE_COMPONENT_LIMIT - 3;
        let message = create_payload(vec![create_post_component(
            &post_with_media(gallery_count * MEDIA_GALLERY_ITEM_LIMIT),
            false,
        )]);
        let components = message["components"][0]["components"].as_array().unwrap();

        assert_eq!(components.len() + 1, MESSAGE_COMPONENT_LIMIT);
        assert_eq!(
            components[1..=gallery_count]
                .iter()
                .map(|gallery| gallery["items"].as_array().unwrap().len())
                .sum::<usize>(),
            370
        );
    }

    fn post_with_media(count: usize) -> Post {
        Post {
            url: "https://www.instagram.com/p/post/".to_owned(),
            text: "post text".to_owned(),
            likes: None,
            reposts: None,
            replies: None,
            author: None,
            media: Media {
                all: (0..count)
                    .map(|index| MediaItem {
                        kind: "image".to_owned(),
                        url: format!("https://example.com/{index}.jpg"),
                    })
                    .collect(),
            },
            provider: "instagram".to_owned(),
            translation: None,
            quote: None,
        }
    }

    #[test]
    fn identifies_the_bot_to_fxembed() {
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

mod post;
mod source;

use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bot_core::command::{Command, CommandContext, CommandFuture};
use bot_core::raw;
use bot_core::response::{Embed, EmbedKind};
use bot_core::{BotState, Error};
use database::UserSocialEmbedsRepository;
use database::settings::TranslationLanguage;
use reqwest::{Client, RequestBuilder, Url, header::USER_AGENT};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use source::Request as EmbedRequest;
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_http::routing::Route;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::command::{
    Command as CommandDefinition, CommandOptionChoice, CommandOptionChoiceValue, CommandType,
};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandOptionValue,
};
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::{
    Interaction, InteractionContextType, InteractionType,
};
use twilight_model::channel::Message;
use twilight_model::channel::message::MessageFlags;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker};
use twilight_model::oauth::ApplicationIntegrationType;
use twilight_util::builder::command::CommandBuilder;
use twilight_util::builder::interaction_response::{AutocompleteBuilder, ChannelMessageBuilder};

type MessageId = Id<MessageMarker>;

pub use source::abembed::{URL_FLAG as ABEMBED_URL_FLAG, set_base_url as set_abembed_url};
pub use source::twitter::primary_translation_language;
pub use source::{SocialDisplay, SocialPlatform};

const MESSAGE_COMPONENT_LIMIT: usize = 40;
const ABEMBED_MEDIA_LIMIT: &str = "380";
const EMBED_API_RETRIES: u32 = 3;
const EMBED_API_RETRY_DELAY: Duration = Duration::from_secs(1);
const EMBEDDED_MESSAGE_CACHE_LIMIT: usize = 100;
const EMBEDDED_MESSAGE_CACHE_TTL: Duration = Duration::from_secs(5 * 60);
const FOLLOWUP_TOKEN_TTL: Duration = Duration::from_secs(15 * 60);
const DEFER_AFTER: Duration = Duration::from_secs(2);
const MAX_CHOICES: usize = 25;
const APP_USER_AGENT: &str = concat!("yin/", env!("CARGO_PKG_VERSION"));
const DISCORD_USER_AGENT: &str = "Discordbot/2.0";
const NOTICE_ID: &str = "social:notice";
static EMBEDDED_MESSAGES: LazyLock<Mutex<VecDeque<EmbeddedMessage>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));
static FOLLOWUP_TOKENS: Mutex<Vec<FollowupToken>> = Mutex::new(Vec::new());
static HTTP: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("valid social embed HTTP client")
});

pub const LANGUAGES: &[(&str, &str)] = &[
    ("English", "en"),
    ("Japanese", "ja"),
    ("Korean", "ko"),
    ("Chinese (Simplified)", "zh-cn"),
    ("Chinese (Traditional)", "zh-tw"),
    ("Spanish", "es"),
    ("Portuguese", "pt"),
    ("French", "fr"),
    ("German", "de"),
    ("Italian", "it"),
    ("Russian", "ru"),
    ("Ukrainian", "uk"),
    ("Polish", "pl"),
    ("Dutch", "nl"),
    ("Turkish", "tr"),
    ("Arabic", "ar"),
    ("Hindi", "hi"),
    ("Indonesian", "id"),
    ("Thai", "th"),
    ("Vietnamese", "vi"),
    ("Filipino", "tl"),
    ("Swedish", "sv"),
    ("Czech", "cs"),
    ("Greek", "el"),
    ("Hebrew", "he"),
    ("Amharic", "am"),
    ("Armenian", "hy"),
    ("Basque", "eu"),
    ("Bengali", "bn"),
    ("Bulgarian", "bg"),
    ("Catalan", "ca"),
    ("Croatian", "hr"),
    ("Danish", "da"),
    ("Estonian", "et"),
    ("Finnish", "fi"),
    ("Georgian", "ka"),
    ("Gujarati", "gu"),
    ("Hungarian", "hu"),
    ("Kannada", "kn"),
    ("Latvian", "lv"),
    ("Lithuanian", "lt"),
    ("Malayalam", "ml"),
    ("Marathi", "mr"),
    ("Norwegian", "no"),
    ("Persian", "fa"),
    ("Romanian", "ro"),
    ("Serbian", "sr"),
    ("Slovak", "sk"),
    ("Slovenian", "sl"),
    ("Tamil", "ta"),
    ("Telugu", "te"),
    ("Urdu", "ur"),
];

pub fn commands() -> Vec<Command> {
    vec![
        Command::slash::<EmbedCommand>(embed).ephemeral(),
        Command::new(refresh_media_definition(), refresh_media).ephemeral(),
    ]
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "embed",
    desc = "Send the social embed for a link",
    contexts = "guild bot_dm private_channel",
    integration_types = "guild_install user_install"
)]
struct EmbedCommand {
    /// Link to embed
    link: String,
    /// Translation language, such as en, ja, pt-BR, or zh-Hant
    #[command(autocomplete = true)]
    language: Option<String>,
    /// Hide the embed behind a spoiler
    spoiler: Option<bool>,
    /// How to display the embed. Defaults to your saved preference
    display: Option<SocialDisplay>,
    /// Text to send with the embed
    #[command(max_length = 2000)]
    text: Option<String>,
}

fn embed(ctx: &CommandContext) -> CommandFuture<'_> {
    Box::pin(async move {
        if ctx.interaction.kind == InteractionType::ApplicationCommandAutocomplete {
            ctx.state
                .http
                .interaction(ctx.interaction.application_id)
                .create_response(
                    ctx.interaction.id,
                    &ctx.interaction.token,
                    &AutocompleteBuilder::new(language_choices(&ctx.data)).build(),
                )
                .await?;
            return Ok(());
        }
        let EmbedCommand {
            link,
            language,
            spoiler,
            display,
            text,
        } = ctx.parse()?;
        let user_id = ctx.author().id;
        let language = match language {
            Some(language) => match TranslationLanguage::parse(&language) {
                Some(language) => Some(language.into_string()),
                None => {
                    return ctx
                        .error("Use a valid ISO language code, such as `en` or `pt-BR`.")
                        .await;
                }
            },
            None => None,
        };
        let work = async {
            let users = UserSocialEmbedsRepository::new(&ctx.state.database);
            let language = match language {
                Some(language) => language,
                None => default_language(
                    users.translation_language(user_id.get()).await?,
                    ctx.interaction.locale.as_deref().unwrap_or_default(),
                ),
            };
            let mut links = source::parse_links_with_alt_fixers(&link);
            if spoiler == Some(true) {
                links.iter_mut().for_each(|link| link.spoiler = true);
            }
            match display {
                Some(display) => links.iter_mut().for_each(|link| link.display = display),
                None => source::apply_displays(&mut links, &users.displays(user_id.get()).await?),
            }
            embed_components(
                &ctx.state,
                links,
                user_id,
                ctx.guild_id(),
                Some(language),
                true,
            )
            .await
        };
        tokio::pin!(work);

        let (result, deferred) = match tokio::time::timeout(DEFER_AFTER, &mut work).await {
            Ok(result) => (result, false),
            Err(_) => {
                ctx.defer().await?;
                (work.await, true)
            }
        };
        let mut components = match result {
            Ok(components) => components,
            Err(error) => {
                let Some(unavailable) = error.downcast_ref::<PostUnavailable>() else {
                    return Err(error);
                };
                tracing::warn!(error = %unavailable, "social post unavailable");
                return ctx.send(unavailable_embed(unavailable)).await;
            }
        };
        if components.is_empty() {
            return ctx.error("That is not a supported social link.").await;
        }
        if let Some(text) = text {
            components.insert(0, json!({"type": 10, "content": text}));
        }
        if deferred {
            components.insert(0, invoker_header(user_id));
        }

        let application_id = ctx.interaction.application_id;
        let token = &ctx.interaction.token;
        let mut batches = component_batches(components)?.into_iter();
        if deferred {
            ctx.state
                .http
                .interaction(application_id)
                .delete_response(token)
                .await?;
        } else if let Some(components) = batches.next() {
            let response = raw::request(
                &ctx.state.http,
                &Route::InteractionCallback {
                    interaction_id: ctx.interaction.id.get(),
                    interaction_token: token,
                    with_response: true,
                },
                Some(&json!({"type": 4, "data": create_payload(components)})),
            )
            .await?;
            ctx.mark_responded();
            remember_followup_token(
                message_id(&response["resource"]["message"])?,
                token,
                Instant::now(),
            );
        }
        for components in batches {
            let followup = raw::request(
                &ctx.state.http,
                &Route::ExecuteWebhook {
                    thread_id: None,
                    token,
                    wait: None,
                    with_components: None,
                    webhook_id: application_id.get(),
                },
                Some(&create_payload(components)),
            )
            .await?;
            remember_followup_token(message_id(&followup)?, token, Instant::now());
        }

        if UserSocialEmbedsRepository::new(&ctx.state.database)
            .mark_notice_shown(user_id.get(), "command")
            .await?
        {
            ctx.send(
                Embed::new(EmbedKind::Info, "Customize your embeds").description(
                    "Use `/settings` to choose how Yin embeds links from each platform.",
                ),
            )
            .await?;
        }
        Ok(())
    })
}

fn language_choices(data: &CommandData) -> Vec<CommandOptionChoice> {
    let partial = data
        .options
        .iter()
        .find_map(|option| match &option.value {
            CommandOptionValue::Focused(value, _) => Some(value.as_str()),
            _ => None,
        })
        .unwrap_or_default();
    matching_languages(partial)
        .take(MAX_CHOICES)
        .map(|(name, code)| CommandOptionChoice {
            name: format!("{name} ({code})"),
            name_localizations: None,
            value: CommandOptionChoiceValue::String(code.to_owned()),
        })
        .collect()
}

fn matching_languages(partial: &str) -> impl Iterator<Item = (&'static str, &'static str)> {
    let partial = partial.trim().to_lowercase();
    LANGUAGES.iter().copied().filter(move |(name, code)| {
        name.to_lowercase().contains(&partial) || code.starts_with(&partial)
    })
}

fn default_language(saved: Option<String>, locale: &str) -> String {
    saved
        .and_then(|language| TranslationLanguage::parse(&language))
        .map(TranslationLanguage::into_string)
        .or_else(|| primary_translation_language(locale))
        .unwrap_or_else(|| "en".to_owned())
}

fn refresh_media_definition() -> CommandDefinition {
    CommandBuilder::new("Refresh Media", "", CommandType::Message)
        .contexts([
            InteractionContextType::Guild,
            InteractionContextType::BotDm,
            InteractionContextType::PrivateChannel,
        ])
        .integration_types([
            ApplicationIntegrationType::GuildInstall,
            ApplicationIntegrationType::UserInstall,
        ])
        .build()
}

fn refresh_media(ctx: &CommandContext) -> CommandFuture<'_> {
    Box::pin(async move {
        let message = ctx
            .data
            .target_id
            .zip(ctx.data.resolved.as_ref())
            .and_then(|(target_id, resolved)| resolved.messages.get(&target_id.cast()))
            .ok_or("message context menu missing target message")?;
        let bot_id = ctx.state.cache.current_user().map(|user| user.id);
        if Some(message.author.id) != bot_id {
            return ctx.error("Only social embeds can be refreshed.").await;
        }
        let application_id = ctx.interaction.application_id.get();
        let token = followup_token(message.id, Instant::now());
        let (get, update) = match &token {
            Some(token) => (
                Route::GetFollowupMessage {
                    application_id,
                    interaction_token: token,
                    thread_id: None,
                    message_id: message.id.get(),
                },
                Route::UpdateWebhookMessage {
                    message_id: message.id.get(),
                    thread_id: None,
                    token,
                    webhook_id: application_id,
                },
            ),
            None => (
                Route::GetMessage {
                    channel_id: message.channel_id.get(),
                    message_id: message.id.get(),
                },
                Route::UpdateMessage {
                    channel_id: message.channel_id.get(),
                    message_id: message.id.get(),
                },
            ),
        };
        let mut message = match raw::request(&ctx.state.http, &get, None).await {
            Ok(message) => message,
            Err(error) => {
                tracing::warn!(error = %error, "failed to fetch social embed for refresh");
                return ctx.error("I can't edit that message anymore.").await;
            }
        };

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        if !refresh_media_urls(&mut message["components"], timestamp) {
            return ctx.error("That message has no media to refresh.").await;
        }

        raw::request(
            &ctx.state.http,
            &update,
            Some(&json!({"components": message["components"]})),
        )
        .await?;

        ctx.send(Embed::new(EmbedKind::Info, "Refreshed the media."))
            .await
    })
}

fn message_id(message: &Value) -> Result<MessageId, serde_json::Error> {
    serde_json::from_value(message["id"].clone())
}

pub async fn handle_message(data: &BotState, message: &Message) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

    let response_ids = match send_embeds(data, message).await {
        Ok(response_ids) if response_ids.is_empty() => return Ok(()),
        Ok(response_ids) => response_ids,
        Err(error) => {
            report_failure(data, message, &error).await;
            return Err(error);
        }
    };

    remember_message(message.id, response_ids, &message.content, Instant::now());
    data.http
        .update_message(message.channel_id, message.id)
        .flags(MessageFlags::SUPPRESS_EMBEDS)
        .await?;
    Ok(())
}

pub async fn handle_interaction(
    data: &BotState,
    interaction: &Interaction,
    component: &MessageComponentInteractionData,
) -> Result<(), Error> {
    if component.custom_id != NOTICE_ID {
        return Ok(());
    }
    let embed = Embed::new(EmbedKind::Info, "Social embeds").description(
        "Yin replaces supported social links with an embed of the post and hides Discord's link \
         preview. Use `/settings` to choose how your links are embedded, or set a platform to \
         **Off** to stop Yin embedding it.",
    );
    let response = ChannelMessageBuilder::new()
        .embeds([embed.into()])
        .flags(MessageFlags::EPHEMERAL)
        .build();
    data.http
        .interaction(interaction.application_id)
        .create_response(interaction.id, &interaction.token, &response)
        .await?;
    Ok(())
}

pub async fn handle_message_delete(
    data: &BotState,
    channel_id: Id<ChannelMarker>,
    message_id: MessageId,
) -> Result<(), Error> {
    for response_id in take_embedded_responses(message_id) {
        data.http.delete_message(channel_id, response_id).await?;
    }
    Ok(())
}

pub async fn handle_message_update(data: &BotState, message: &Message) -> Result<(), Error> {
    if !should_refresh_message(message.id, &message.content) {
        return Ok(());
    }

    let response_ids = match send_embeds(data, message).await {
        Ok(response_ids) => response_ids,
        Err(error) => {
            report_failure(data, message, &error).await;
            return Err(error);
        }
    };
    let suppress_embeds = !response_ids.is_empty();

    let old_response_ids = take_embedded_responses(message.id);
    remember_message(message.id, response_ids, &message.content, Instant::now());
    for response_id in old_response_ids {
        data.http
            .delete_message(message.channel_id, response_id)
            .await?;
    }
    if suppression_changed(
        message.flags.unwrap_or(MessageFlags::empty()),
        suppress_embeds,
    ) {
        let flags = if suppress_embeds {
            MessageFlags::SUPPRESS_EMBEDS
        } else {
            MessageFlags::empty()
        };
        data.http
            .update_message(message.channel_id, message.id)
            .flags(flags)
            .await?;
    }
    Ok(())
}

async fn report_failure(data: &BotState, message: &Message, error: &Error) {
    let _ = data
        .http
        .create_reaction(
            message.channel_id,
            message.id,
            &RequestReactionType::Unicode { name: "❌" },
        )
        .await;
    let link = format!(
        "https://discord.com/channels/{}/{}/{}",
        message
            .guild_id
            .map_or_else(|| "@me".to_owned(), |id| id.to_string()),
        message.channel_id,
        message.id
    );
    let embed = match error.downcast_ref::<PostUnavailable>() {
        Some(unavailable) => unavailable_embed(unavailable),
        None => Embed::new(
            EmbedKind::Error,
            "Something went wrong while embedding that post.",
        ),
    }
    .field("Message", link, false);
    let sent = async {
        let channel = data
            .http
            .create_private_channel(message.author.id)
            .await?
            .model()
            .await?;
        data.http
            .create_message(channel.id)
            .embeds(&[embed.into()])
            .await?;
        Ok::<_, Error>(())
    }
    .await;
    if let Err(error) = sent {
        tracing::warn!(error = %error, "failed to DM social embed failure");
    }
}

fn unavailable_embed(unavailable: &PostUnavailable) -> Embed {
    let embed = Embed::new(EmbedKind::Error, "That post could not be embedded.");
    match &unavailable.reason {
        Some(reason) => embed.description(reason),
        None => embed,
    }
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

struct FollowupToken {
    created_at: Instant,
    message_id: MessageId,
    token: String,
}

fn remember_followup_token(message_id: MessageId, token: &str, now: Instant) {
    let mut tokens = FOLLOWUP_TOKENS.lock().unwrap();
    tokens.retain(|token| now.saturating_duration_since(token.created_at) < FOLLOWUP_TOKEN_TTL);
    tokens.push(FollowupToken {
        created_at: now,
        message_id,
        token: token.to_owned(),
    });
}

fn followup_token(message_id: MessageId, now: Instant) -> Option<String> {
    FOLLOWUP_TOKENS
        .lock()
        .unwrap()
        .iter()
        .find(|token| {
            token.message_id == message_id
                && now.saturating_duration_since(token.created_at) < FOLLOWUP_TOKEN_TTL
        })
        .map(|token| token.token.clone())
}

fn suppression_changed(flags: MessageFlags, suppress: bool) -> bool {
    flags.contains(MessageFlags::SUPPRESS_EMBEDS) != suppress
}

async fn send_embeds(data: &BotState, message: &Message) -> Result<Vec<MessageId>, Error> {
    let mut links = source::parse_links(&message.content);
    if links.is_empty() {
        return Ok(Vec::new());
    }
    let repository = UserSocialEmbedsRepository::new(&data.database);
    let disabled_platforms = repository
        .effective_disabled_platforms(message.author.id.get(), message.guild_id.map(|id| id.get()))
        .await?;
    source::retain_enabled_links(&mut links, &disabled_platforms);
    if links.is_empty() {
        return Ok(Vec::new());
    }
    source::apply_displays(
        &mut links,
        &repository.displays(message.author.id.get()).await?,
    );
    let mut components = embed_components(
        data,
        links,
        message.author.id,
        message.guild_id,
        None,
        false,
    )
    .await?;
    if !components.is_empty()
        && repository
            .mark_notice_shown(message.author.id.get(), "auto")
            .await?
    {
        components.push(json!({
            "type": 1,
            "components": [{"type": 2, "style": 2, "label": "What's this?", "custom_id": NOTICE_ID}],
        }));
    }

    let mut response_ids = Vec::new();
    for components in component_batches(components)? {
        response_ids.push(send_payload(data, message, create_payload(components)).await?);
    }
    Ok(response_ids)
}

async fn embed_components(
    data: &BotState,
    links: Vec<source::Link>,
    user_id: Id<UserMarker>,
    guild_id: Option<Id<GuildMarker>>,
    language: Option<String>,
    view_link: bool,
) -> Result<Vec<Value>, Error> {
    let twitter = if links
        .iter()
        .any(|link| link.platform == SocialPlatform::Twitter)
    {
        source::twitter::options(data, user_id, guild_id, language).await?
    } else {
        source::twitter::Options::default()
    };

    let abembed =
        source::abembed::base_url(&data.database, user_id.get(), guild_id.map(Id::get)).await?;

    let spec = if links
        .iter()
        .any(|link| source::abembed::platform(&link.url).is_some())
    {
        source::abembed::spec(&data.database).await
    } else {
        None
    };

    let mut components = Vec::new();
    for link in links {
        let platform = link.platform;
        let display = link.display;
        let view_link = view_link.then(|| {
            json!({
                "type": 1,
                "components": [{
                    "type": 2,
                    "style": 5,
                    "label": format!("View on {}", platform.display_name()),
                    "emoji": {"name": "🔗"},
                    "url": link.url.as_str(),
                }],
            })
        });
        let Some(request) = link.request(&twitter, &abembed, spec.as_ref()) else {
            continue;
        };
        let mut component = match request {
            EmbedRequest::FxEmbed { url, spoiler } => {
                post::component(&fetch_json::<post::Response>(&url).await?.status, spoiler)
            }
            EmbedRequest::AbEmbed { url, spoiler } => {
                let url = abembed_component_url(&url)?;
                let mut component = fetch_json::<AbEmbedComponent>(url.as_str())
                    .await?
                    .component;
                component["spoiler"] = json!(spoiler);
                if platform == SocialPlatform::Twitter {
                    rewrite_abembed_gifs(&mut component);
                }
                component
            }
            EmbedRequest::Component { url, spoiler } => fetch_component(&url, spoiler).await?,
        };
        let media_only = display == SocialDisplay::MediaOnly && keep_only_media(&mut component);
        components.push(component);
        if media_only {
            components.extend(view_link);
        }
    }
    Ok(components)
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
    data: &BotState,
    message: &Message,
    mut payload: Value,
) -> Result<MessageId, Error> {
    payload["message_reference"] = json!({"message_id": message.id});
    payload["allowed_mentions"]["replied_user"] = json!(false);
    let response = raw::request(
        &data.http,
        &Route::CreateMessage {
            channel_id: message.channel_id.get(),
        },
        Some(&payload),
    )
    .await?;
    Ok(message_id(&response)?)
}

#[derive(Deserialize)]
struct AbEmbedComponent {
    component: Value,
}

fn abembed_component_url(url: &str) -> Result<Url, Error> {
    Ok(Url::parse_with_params(
        url,
        [("media_limit", ABEMBED_MEDIA_LIMIT)],
    )?)
}

async fn fetch_json<T: DeserializeOwned>(api_url: &str) -> Result<T, Error> {
    for retry in 0..=EMBED_API_RETRIES {
        match try_fetch_json(api_url).await {
            Ok(body) => return Ok(body),
            Err(error) if retry < EMBED_API_RETRIES && !error.is::<PostUnavailable>() => {
                tokio::time::sleep(EMBED_API_RETRY_DELAY * 2u32.pow(retry)).await;
            }
            Err(error) => return Err(error),
        }
    }

    unreachable!()
}

async fn try_fetch_json<T: DeserializeOwned>(api_url: &str) -> Result<T, Error> {
    let response = request(api_url).send().await?;
    let status = response.status();
    if status.is_client_error() {
        let body = response.text().await.unwrap_or_default();
        return Err(PostUnavailable {
            status,
            reason: api_error_reason(&body),
        }
        .into());
    }
    Ok(response.error_for_status()?.json().await?)
}

fn api_error_reason(body: &str) -> Option<String> {
    serde_json::from_str::<Value>(body)
        .ok()?
        .get("error")?
        .as_str()
        .map(str::to_owned)
}

#[derive(Debug)]
struct PostUnavailable {
    status: reqwest::StatusCode,
    reason: Option<String>,
}

impl std::fmt::Display for PostUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "embed API returned {}", self.status)?;
        if let Some(reason) = &self.reason {
            write!(f, ": {reason}")?;
        }
        Ok(())
    }
}

impl std::error::Error for PostUnavailable {}

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

fn refresh_media_urls(value: &mut Value, timestamp: u64) -> bool {
    let mut refreshed = false;
    match value {
        Value::Array(values) => {
            for value in values {
                refreshed |= refresh_media_urls(value, timestamp);
            }
        }
        Value::Object(values) => {
            let url = values
                .get("media")
                .and_then(|media| media["url"].as_str())
                .map(|url| cache_busted(url, timestamp));
            if let Some(url) = url {
                values.insert("media".to_owned(), json!({"url": url}));
                refreshed = true;
            }
            for value in values.values_mut() {
                refreshed |= refresh_media_urls(value, timestamp);
            }
        }
        _ => {}
    }
    refreshed
}

fn cache_busted(url: &str, timestamp: u64) -> String {
    let Ok(mut url) = Url::parse(url) else {
        return url.to_owned();
    };
    let query = url
        .query_pairs()
        .filter(|(key, _)| key != "t")
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    url.query_pairs_mut()
        .clear()
        .extend_pairs(query)
        .append_pair("t", &timestamp.to_string());
    url.into()
}

fn keep_only_media(component: &mut Value) -> bool {
    let Some(children) = component
        .get_mut("components")
        .and_then(Value::as_array_mut)
    else {
        return false;
    };
    let is_media = |child: &Value| child["type"] == 12;
    if !children.iter().any(is_media) {
        return false;
    }
    children.retain(is_media);
    true
}

fn invoker_header(user_id: Id<UserMarker>) -> Value {
    json!({"type": 10, "content": format!("-# <@{user_id}> used `/embed`")})
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
    fn default_language_prefers_saved_language_then_client_locale() {
        assert_eq!(default_language(Some("ja".to_owned()), "en-US"), "ja");
        assert_eq!(default_language(None, "pt-BR"), "pt");
        assert_eq!(default_language(Some("not valid".to_owned()), "fr"), "fr");
        assert_eq!(default_language(None, ""), "en");
    }

    #[test]
    fn finds_multiple_supported_links_in_message_order() {
        let twitter = source::twitter::Options::default();
        let route = |response: &str| {
            json!({"get": {"responses": {"200": {
                "$ref": format!("#/components/responses/{response}"),
            }}}})
        };
        let spec = source::abembed::Spec::parse(
            &json!({"paths": {
                "/tiktok/{shortcode}": route("DiscordEmbed"),
                "/api/tiktok/{shortcode}": route("Post"),
            }})
            .to_string(),
        )
        .unwrap();
        let requests = source::parse_links(concat!(
            "first https://x.com/jack/status/20 ",
            "then ||https://open.spotify.com/track/11dFghVXANMlKmJXsNCbNl|| ",
            "and https://vm.tiktok.com/ZN88Qw7ns/"
        ))
        .into_iter()
        .filter_map(|link| link.request(&twitter, "https://abembed.com/", Some(&spec)))
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
                    url: "https://abembed.com/api/tiktok/ZN88Qw7ns/component".to_owned(),
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
    fn media_only_keeps_galleries_and_falls_back_when_there_is_no_media() {
        let mut with_media = json!({
            "type": 17,
            "spoiler": true,
            "components": [
                {"type": 10, "content": "text"},
                {"type": 12, "items": [{"media": {"url": "https://example.com/a.jpg"}}]},
                {"type": 14},
                {"type": 12, "items": [{"media": {"url": "https://example.com/b.jpg"}}]},
            ],
        });
        let text_only = json!({"type": 17, "components": [{"type": 10, "content": "text"}]});
        let mut without_media = text_only.clone();

        assert!(keep_only_media(&mut with_media));
        assert!(!keep_only_media(&mut without_media));

        assert_eq!(with_media["spoiler"], true);
        assert_eq!(
            with_media["components"]
                .as_array()
                .unwrap()
                .iter()
                .map(|child| child["type"].clone())
                .collect::<Vec<_>>(),
            [12, 12]
        );
        assert_eq!(without_media, text_only);
    }

    #[test]
    fn remembers_followup_tokens_until_discord_expires_them() {
        let now = Instant::now();
        remember_followup_token(MessageId::new(500), "token", now);

        assert_eq!(
            followup_token(MessageId::new(500), now).as_deref(),
            Some("token")
        );
        assert_eq!(followup_token(MessageId::new(501), now), None);
        assert_eq!(
            followup_token(MessageId::new(500), now + FOLLOWUP_TOKEN_TTL),
            None
        );
    }

    #[test]
    fn refreshes_every_media_url_with_the_timestamp() {
        let mut components = json!([{
            "type": 17,
            "components": [
                {"type": 10, "content": "https://example.com/text.jpg"},
                {"type": 9, "accessory": {"type": 11, "media": {"url": "https://example.com/a.jpg", "proxy_url": "https://media.discordapp.net/a.jpg"}}},
                {"type": 12, "items": [{"media": {"url": "https://example.com/b.mp4?tag=1&t=5"}, "spoiler": true}]},
            ],
        }]);
        let mut text_only = json!([{"type": 17, "components": [{"type": 10, "content": "text"}]}]);

        assert!(refresh_media_urls(&mut components, 9));
        assert!(!refresh_media_urls(&mut text_only, 9));
        let children = &components[0]["components"];
        assert_eq!(children[0]["content"], "https://example.com/text.jpg");
        assert_eq!(
            children[1]["accessory"]["media"],
            json!({"url": "https://example.com/a.jpg?t=9"})
        );
        assert_eq!(
            children[2]["items"][0],
            json!({"media": {"url": "https://example.com/b.mp4?tag=1&t=9"}, "spoiler": true})
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
        assert!(!suppression_changed(MessageFlags::SUPPRESS_EMBEDS, true,));
        assert!(suppression_changed(MessageFlags::empty(), true));
        assert!(suppression_changed(MessageFlags::SUPPRESS_EMBEDS, false,));
    }

    #[test]
    fn asks_abembed_for_as_much_media_as_a_message_fits() {
        for (url, limited) in [
            (
                "https://abembed.com/api/tiktok/ZN88Qw7ns/component",
                "https://abembed.com/api/tiktok/ZN88Qw7ns/component?media_limit=380",
            ),
            (
                "https://abembed.com/api/facebook/photo/component?fbid=1&set=a.2",
                "https://abembed.com/api/facebook/photo/component?fbid=1&set=a.2&media_limit=380",
            ),
        ] {
            assert_eq!(abembed_component_url(url).unwrap().as_str(), limited);
        }
    }

    #[test]
    fn identifies_the_bot_to_embed_apis() {
        let request = request("https://api.fxtwitter.com/2/status/20")
            .build()
            .unwrap();

        assert_eq!(request.headers()[USER_AGENT], APP_USER_AGENT);
    }

    #[test]
    fn reads_the_reason_from_embed_api_errors() {
        assert_eq!(
            api_error_reason(r#"{"error":"Age-restricted content"}"#).as_deref(),
            Some("Age-restricted content")
        );
        assert_eq!(api_error_reason(r#"{"code":404}"#), None);
        assert_eq!(api_error_reason("<html></html>"), None);
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
            message_id: MessageId::new(1),
            response_ids: vec![MessageId::new(101)],
            content: "expired".to_owned(),
        });
        drop(messages);

        for id in 2..=EMBEDDED_MESSAGE_CACHE_LIMIT as u64 + 2 {
            remember_message(
                MessageId::new(id),
                vec![MessageId::new(id + 100)],
                &format!("content {id}"),
                now,
            );
        }

        let messages = EMBEDDED_MESSAGES.lock().unwrap();
        assert_eq!(messages.len(), EMBEDDED_MESSAGE_CACHE_LIMIT);
        assert_eq!(messages.front().unwrap().message_id, MessageId::new(3));
        assert_eq!(messages.back().unwrap().message_id, MessageId::new(102));
        drop(messages);

        assert!(!should_refresh_message(MessageId::new(3), "content 3"));
        assert!(should_refresh_message(MessageId::new(3), "edited content"));
        assert_eq!(
            take_embedded_responses(MessageId::new(3)),
            vec![MessageId::new(103)]
        );
        assert!(take_embedded_responses(MessageId::new(3)).is_empty());
        assert_eq!(
            take_embedded_responses(MessageId::new(102)),
            vec![MessageId::new(202)]
        );

        remember_message(
            MessageId::new(200),
            vec![MessageId::new(300), MessageId::new(301)],
            "two embeds",
            now,
        );
        assert_eq!(
            take_embedded_responses(MessageId::new(200)),
            [MessageId::new(300), MessageId::new(301)]
        );

        EMBEDDED_MESSAGES.lock().unwrap().clear();
    }
}

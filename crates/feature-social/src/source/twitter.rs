use bot_core::serenity;
use bot_core::{BotState, Error};
use database::settings::TranslationLanguage;
use feature_flags::FlagValue;
use reqwest::Url;

use super::{EmbedSource, Request};

const FXTWITTER_API: &str = "https://api.fxtwitter.com/2/status/";
const ABEMBED_TWITTER: &str = "https://staging.abembed.com/twitter/";
const BACKEND_FLAG: &str = "twitter-embed-backend";
const ABEMBED_VARIANT: &str = "abembed";
const HOSTS: &[&str] = &[
    "x.com",
    "www.x.com",
    "twitter.com",
    "www.twitter.com",
    "mobile.twitter.com",
];

const DISPLAY_NAME: &str = "X / Twitter";
const ACCENT_COLOR: u32 = 0x1d9bf0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Twitter;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Backend {
    #[default]
    FxTwitter,
    AbEmbed,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Options {
    language: String,
    backend: Backend,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            language: "en".to_owned(),
            backend: Backend::default(),
        }
    }
}

pub(crate) async fn options(
    data: &BotState,
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<Options, Error> {
    Ok(Options {
        language: guild_translation_language(data, ctx, message).await?,
        backend: selected_backend(data, message).await,
    })
}

impl EmbedSource for Twitter {
    fn handles(&self, url: &Url) -> bool {
        post_parts(url).is_some()
    }

    fn request(&self, url: &Url, options: &Options, spoiler: bool) -> Option<Request> {
        let (username, id, language) = post_parts(url)?;
        let language = language
            .and_then(TranslationLanguage::parse)
            .map(TranslationLanguage::into_string)
            .unwrap_or_else(|| options.language.clone());

        Some(match options.backend {
            Backend::FxTwitter => Request::FxEmbed {
                url: format!("{FXTWITTER_API}{id}?lang={language}"),
                spoiler,
            },
            Backend::AbEmbed => Request::Component {
                url: format!("{ABEMBED_TWITTER}{username}/status/{id}/{language}"),
                spoiler,
            },
        })
    }

    fn display_name(&self) -> &'static str {
        DISPLAY_NAME
    }

    fn accent_color(&self) -> u32 {
        ACCENT_COLOR
    }

    fn media_url(&self, _post_url: &str, kind: &str, media_url: &str) -> Option<String> {
        if kind != "gif" {
            return None;
        }

        let mut url = Url::parse(media_url).ok()?;
        if url.host_str() != Some("video.twimg.com") || !url.path().ends_with(".mp4") {
            return None;
        }
        let path = url.path().trim_end_matches(".mp4").to_owned() + ".gif";
        url.set_host(Some("gif.fxtwitter.com")).ok()?;
        url.set_path(&path);
        Some(url.into())
    }
}

pub fn primary_translation_language(locale: &str) -> Option<String> {
    TranslationLanguage::parse(locale.split(['-', '_']).next()?)
        .map(TranslationLanguage::into_string)
}

fn post_parts(url: &Url) -> Option<(&str, &str, Option<&str>)> {
    let host = url.host_str()?;
    let parts: Vec<_> = url.path_segments()?.collect();
    match parts.as_slice() {
        [username, "status", id, rest @ ..]
            if HOSTS.contains(&host)
                && (2..=20).contains(&id.len())
                && id.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            Some((username, id, rest.first().copied()))
        }
        _ => None,
    }
}

async fn selected_backend(data: &BotState, message: &serenity::Message) -> Backend {
    match data
        .feature_flags
        .value(
            BACKEND_FLAG,
            message.author.id.get(),
            message.guild_id.map(|id| id.get()),
        )
        .await
    {
        Ok(value) => backend_from_flag(value),
        Err(error) => {
            tracing::warn!(error = %error, "failed to evaluate Twitter embed backend flag");
            Backend::FxTwitter
        }
    }
}

fn backend_from_flag(value: Option<FlagValue>) -> Backend {
    match value {
        Some(FlagValue::String(value)) if value == ABEMBED_VARIANT => Backend::AbEmbed,
        _ => Backend::FxTwitter,
    }
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
        .and_then(|language| TranslationLanguage::parse(&language))
        .map(TranslationLanguage::into_string);
    Ok(configured.unwrap_or(default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_backend_specific_requests_and_normalizes_languages() {
        assert_eq!(
            backend_from_flag(Some(FlagValue::String(ABEMBED_VARIANT.to_owned()))),
            Backend::AbEmbed
        );
        assert_eq!(
            backend_from_flag(Some(FlagValue::Boolean(true))),
            Backend::FxTwitter
        );
        let default_language_url = Url::parse("https://x.com/jack/status/20?s=20").unwrap();
        assert_eq!(
            Twitter.request(
                &default_language_url,
                &Options {
                    language: "fr".to_owned(),
                    backend: Backend::FxTwitter,
                },
                false,
            ),
            Some(Request::FxEmbed {
                url: "https://api.fxtwitter.com/2/status/20?lang=fr".to_owned(),
                spoiler: false,
            })
        );
        let url = Url::parse("https://x.com/jack/status/20/jp?s=20").unwrap();
        assert_eq!(
            Twitter.request(
                &url,
                &Options {
                    language: "fr".to_owned(),
                    backend: Backend::FxTwitter,
                },
                false,
            ),
            Some(Request::FxEmbed {
                url: "https://api.fxtwitter.com/2/status/20?lang=ja".to_owned(),
                spoiler: false,
            })
        );
        assert_eq!(
            Twitter.request(
                &url,
                &Options {
                    language: "fr".to_owned(),
                    backend: Backend::AbEmbed,
                },
                true,
            ),
            Some(Request::Component {
                url: "https://staging.abembed.com/twitter/jack/status/20/ja".to_owned(),
                spoiler: true,
            })
        );
        assert_eq!(primary_translation_language("pt-BR").as_deref(), Some("pt"));
        assert!(!Twitter.handles(&Url::parse("https://x.com.example/jack/status/20").unwrap()));
    }

    #[test]
    fn rewrites_only_fxtwitter_gif_media() {
        assert_eq!(
            Twitter
                .media_url(
                    "",
                    "gif",
                    "https://video.twimg.com/tweet_video/animation.mp4"
                )
                .as_deref(),
            Some("https://gif.fxtwitter.com/tweet_video/animation.gif")
        );
        assert!(
            Twitter
                .media_url("", "video", "https://video.twimg.com/video.mp4")
                .is_none()
        );
    }
}

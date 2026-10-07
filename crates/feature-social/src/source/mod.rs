use bot_core::{BotState, serenity};
use reqwest::Url;

mod bluesky;
mod facebook;
mod instagram;
mod spotify;
mod tiktok;
pub(crate) mod twitter;

const ABEMBED: &str = "https://abembed.com/";
const ABEMBED_STAGING: &str = "https://staging.abembed.com/";

#[derive(Clone, Copy, Debug, Eq, PartialEq, poise::ChoiceParameter)]
pub enum SocialPlatform {
    #[name = "twitter"]
    #[name = "x"]
    Twitter,
    #[name = "bluesky"]
    Bluesky,
    #[name = "instagram"]
    Instagram,
    #[name = "facebook"]
    Facebook,
    #[name = "tiktok"]
    TikTok,
    #[name = "spotify"]
    Spotify,
}

impl SocialPlatform {
    pub const ALL: [Self; 6] = [
        Self::Twitter,
        Self::Bluesky,
        Self::Instagram,
        Self::Facebook,
        Self::TikTok,
        Self::Spotify,
    ];

    pub fn key(self) -> &'static str {
        poise::ChoiceParameter::name(&self)
    }

    pub(crate) fn from_provider(provider: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|platform| platform.key() == provider)
    }

    fn recognize(url: &Url) -> Option<Self> {
        [
            Self::Spotify,
            Self::Twitter,
            Self::Bluesky,
            Self::Instagram,
            Self::Facebook,
            Self::TikTok,
        ]
        .into_iter()
        .find(|platform| platform.handles(url))
    }

    fn handles(self, url: &Url) -> bool {
        match self {
            Self::Twitter => twitter::handles(url),
            Self::Bluesky => bluesky::handles(url),
            Self::Instagram => instagram::handles(url),
            Self::Facebook => facebook::handles(url),
            Self::TikTok => tiktok::handles(url),
            Self::Spotify => spotify::handles(url),
        }
    }

    fn request(
        self,
        url: &Url,
        twitter_options: &twitter::Options,
        abembed: &str,
        spoiler: bool,
    ) -> Option<Request> {
        match self {
            Self::Twitter => twitter::request(url, twitter_options, abembed, spoiler),
            Self::Bluesky => bluesky::request(url, spoiler),
            Self::Instagram => instagram::request(url, abembed, spoiler),
            Self::Facebook => facebook::request(url, abembed, spoiler),
            Self::TikTok => tiktok::request(url, abembed, spoiler),
            Self::Spotify => spotify::request(url, spoiler),
        }
    }

    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::Twitter => "X / Twitter",
            Self::Bluesky => "Bluesky",
            Self::Instagram => "Instagram",
            Self::Facebook => "Facebook",
            Self::TikTok => "TikTok",
            Self::Spotify => "Spotify",
        }
    }

    pub(crate) fn accent_color(self) -> u32 {
        match self {
            Self::Bluesky => 0x1185fe,
            Self::Instagram => 0xce0071,
            Self::Facebook => 0x1877f2,
            Self::Twitter | Self::TikTok | Self::Spotify => 0x1d9bf0,
        }
    }

    pub(crate) fn media_url(self, post_url: &str, kind: &str, url: &str) -> Option<String> {
        match self {
            Self::Twitter => twitter::media_url(kind, url),
            Self::Bluesky => bluesky::media_url(post_url, kind),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Request {
    FxEmbed { url: String, spoiler: bool },
    AbEmbed { url: String, spoiler: bool },
    Component { url: String, spoiler: bool },
}

#[derive(Debug, PartialEq)]
pub(crate) struct Link {
    pub(crate) platform: SocialPlatform,
    url: Url,
    spoiler: bool,
}

impl Link {
    pub(crate) fn request(
        self,
        twitter_options: &twitter::Options,
        abembed: &str,
    ) -> Option<Request> {
        self.platform
            .request(&self.url, twitter_options, abembed, self.spoiler)
    }
}

pub(crate) async fn abembed_url(
    data: &BotState,
    message: &serenity::Message,
    platform: SocialPlatform,
) -> &'static str {
    if matches!(platform, SocialPlatform::Bluesky | SocialPlatform::Spotify) {
        return ABEMBED;
    }
    let key = format!("{}-staging-abembed", platform.key());
    match data
        .feature_flags
        .is_enabled(
            &key,
            message.author.id.get(),
            message.guild_id.map(|id| id.get()),
        )
        .await
    {
        Ok(staging) => abembed_from_flag(staging),
        Err(error) => {
            tracing::warn!(error = %error, flag = key, "failed to evaluate abembed staging flag");
            ABEMBED
        }
    }
}

fn abembed_from_flag(staging: bool) -> &'static str {
    if staging { ABEMBED_STAGING } else { ABEMBED }
}

pub(crate) fn parse_links(content: &str) -> Vec<Link> {
    content
        .split_whitespace()
        .filter_map(|word| {
            let (url, spoiler) = parse_url(word)?;
            Some(Link {
                platform: SocialPlatform::recognize(&url)?,
                url,
                spoiler,
            })
        })
        .collect()
}

pub(crate) fn retain_enabled_links(links: &mut Vec<Link>, disabled_platforms: &[String]) {
    links.retain(|link| {
        !disabled_platforms
            .iter()
            .any(|platform| platform == link.platform.key())
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staging_flag_selects_the_abembed_host() {
        assert_eq!(abembed_from_flag(true), "https://staging.abembed.com/");
        assert_eq!(abembed_from_flag(false), "https://abembed.com/");
    }

    #[test]
    fn every_supported_platform_has_an_independent_toggle() {
        for platform in SocialPlatform::ALL {
            assert_eq!(
                SocialPlatform::from_provider(platform.key()),
                Some(platform)
            );
            let mut links = vec![Link {
                platform,
                url: Url::parse("https://example.com").unwrap(),
                spoiler: false,
            }];
            retain_enabled_links(&mut links, &[platform.key().to_owned()]);
            assert!(links.is_empty());
        }
    }

    #[test]
    fn parses_supported_links_in_message_order_and_preserves_spoilers() {
        let links = parse_links(concat!(
            "first https://x.com/jack/status/20 ",
            "then ||<https://open.spotify.com/track/11dFghVXANMlKmJXsNCbNl>|| ",
            "and https://vm.tiktok.com/ZN88Qw7ns/"
        ));

        assert_eq!(
            links.iter().map(|link| link.platform).collect::<Vec<_>>(),
            [
                SocialPlatform::Twitter,
                SocialPlatform::Spotify,
                SocialPlatform::TikTok,
            ]
        );
        assert!(!links[0].spoiler);
        assert!(links[1].spoiler);
        assert!(!links[2].spoiler);
    }
}

use reqwest::Url;
use twilight_interactions::command::{CommandOption, CreateOption};

pub(crate) mod abembed;
mod bluesky;
mod spotify;
pub(crate) mod twitter;

#[derive(Clone, Copy, Debug, Eq, PartialEq, CommandOption, CreateOption)]
pub enum SocialPlatform {
    #[option(name = "twitter", value = "twitter")]
    Twitter,
    #[option(name = "bluesky", value = "bluesky")]
    Bluesky,
    #[option(name = "instagram", value = "instagram")]
    Instagram,
    #[option(name = "facebook", value = "facebook")]
    Facebook,
    #[option(name = "tiktok", value = "tiktok")]
    TikTok,
    #[option(name = "reddit", value = "reddit")]
    Reddit,
    #[option(name = "spotify", value = "spotify")]
    Spotify,
}

impl SocialPlatform {
    pub const ALL: [Self; 7] = [
        Self::Twitter,
        Self::Bluesky,
        Self::Instagram,
        Self::Facebook,
        Self::TikTok,
        Self::Reddit,
        Self::Spotify,
    ];

    pub fn key(self) -> &'static str {
        self.value()
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
            Self::Reddit,
        ]
        .into_iter()
        .find(|platform| platform.handles(url))
    }

    fn handles(self, url: &Url) -> bool {
        match self {
            Self::Twitter => twitter::handles(url),
            Self::Bluesky => bluesky::handles(url),
            Self::Spotify => spotify::handles(url),
            _ => abembed::platform(url) == Some(self),
        }
    }

    fn request(
        self,
        url: &Url,
        twitter_options: &twitter::Options,
        abembed: &str,
        spec: Option<&abembed::Spec>,
        spoiler: bool,
    ) -> Option<Request> {
        match self {
            Self::Twitter => twitter::request(url, twitter_options, abembed, spoiler),
            Self::Bluesky => bluesky::request(url, spoiler),
            Self::Spotify => spotify::request(url, spoiler),
            _ => Some(Request::AbEmbed {
                url: spec?.component_url(url, abembed)?,
                spoiler,
            }),
        }
    }

    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::Twitter => "X / Twitter",
            Self::Bluesky => "Bluesky",
            Self::Instagram => "Instagram",
            Self::Facebook => "Facebook",
            Self::TikTok => "TikTok",
            Self::Reddit => "Reddit",
            Self::Spotify => "Spotify",
        }
    }

    pub(crate) fn accent_color(self) -> u32 {
        match self {
            Self::Bluesky => 0x1185fe,
            _ => 0x1d9bf0,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, CommandOption, CreateOption)]
pub enum SocialDisplay {
    #[default]
    #[option(name = "full", value = "full")]
    Full,
    #[option(name = "media-only", value = "media-only")]
    MediaOnly,
}

impl SocialDisplay {
    pub fn key(self) -> &'static str {
        self.value()
    }

    pub fn from_key(key: &str) -> Option<Self> {
        [Self::Full, Self::MediaOnly]
            .into_iter()
            .find(|display| display.key() == key)
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
    pub(crate) url: Url,
    pub(crate) spoiler: bool,
    pub(crate) display: SocialDisplay,
}

impl Link {
    pub(crate) fn request(
        self,
        twitter_options: &twitter::Options,
        abembed: &str,
        spec: Option<&abembed::Spec>,
    ) -> Option<Request> {
        self.platform
            .request(&self.url, twitter_options, abembed, spec, self.spoiler)
    }
}

const ALT_FIXERS: &[(&str, &str)] = &[
    // abembed
    ("abtwitter.com", "twitter.com"),
    ("abfixx.com", "x.com"),
    ("abinstagram.com", "instagram.com"),
    ("abtiktok.com", "tiktok.com"),
    ("abreddit.com", "reddit.com"),
    ("abfacebook.com", "facebook.com"),
    ("abspotify.com", "spotify.com"),
    ("spqtify.com", "spotify.com"),
    // other fixers
    ("fxtwitter.com", "twitter.com"),
    ("fixupx.com", "x.com"),
    ("vxtwitter.com", "twitter.com"),
    ("fixvx.com", "x.com"),
    ("twittpr.com", "twitter.com"),
    ("ddinstagram.com", "instagram.com"),
    ("kkinstagram.com", "instagram.com"),
    ("instagramez.com", "instagram.com"),
    ("fxstagram.com", "instagram.com"),
    ("vxinstagram.com", "instagram.com"),
    ("rxddit.com", "reddit.com"),
    ("vxreddit.com", "reddit.com"),
    ("redditez.com", "reddit.com"),
    ("facebed.com", "facebook.com"),
    ("facebookez.com", "facebook.com"),
    ("fxspotify.com", "spotify.com"),
    ("vxtiktok.com", "tiktok.com"),
    ("tnktok.com", "tiktok.com"),
    ("tiktxk.com", "tiktok.com"),
    ("tfxktok.com", "tiktok.com"),
    ("kktiktok.com", "tiktok.com"),
    ("tiktokez.com", "tiktok.com"),
];

pub(crate) fn parse_links(content: &str) -> Vec<Link> {
    links(content, false)
}

pub(crate) fn parse_links_with_alt_fixers(content: &str) -> Vec<Link> {
    links(content, true)
}

pub(crate) fn video_links(content: &str) -> Vec<(Url, bool)> {
    content
        .split_whitespace()
        .filter(|word| link(word, true).is_none())
        .filter_map(parse_url)
        .filter(|(url, _)| {
            matches!(url.scheme(), "https") && url.path().to_ascii_lowercase().ends_with(".mp4")
        })
        .collect()
}

fn links(content: &str, alt_fixers: bool) -> Vec<Link> {
    content
        .split_whitespace()
        .filter_map(|word| link(word, alt_fixers))
        .collect()
}

fn link(word: &str, alt_fixers: bool) -> Option<Link> {
    let (mut url, spoiler) = parse_url(word)?;
    if alt_fixers && let Some(host) = native_host(&url) {
        url.set_host(Some(&host)).ok()?;
    }
    Some(Link {
        platform: SocialPlatform::recognize(&url)?,
        url,
        spoiler,
        display: SocialDisplay::Full,
    })
}

pub(crate) fn apply_displays(links: &mut [Link], displays: &[(String, String)]) {
    for link in links {
        link.display = displays
            .iter()
            .find(|(platform, _)| platform == link.platform.key())
            .and_then(|(_, display)| SocialDisplay::from_key(display))
            .unwrap_or_default();
    }
}

pub(crate) fn retain_enabled_links(links: &mut Vec<Link>, disabled_platforms: &[String]) {
    links.retain(|link| {
        !disabled_platforms
            .iter()
            .any(|platform| platform == link.platform.key())
    });
}

fn native_host(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    ALT_FIXERS.iter().find_map(|(fixer, native)| {
        let subdomain = host.strip_suffix(fixer)?;
        (subdomain.is_empty() || subdomain.ends_with('.')).then(|| format!("{subdomain}{native}"))
    })
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
                display: SocialDisplay::Full,
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

    #[test]
    fn alt_fixer_links_become_platform_links_only_when_asked() {
        for (fixer, native) in ALT_FIXERS {
            let (host, path, platform) = match *native {
                "twitter.com" | "x.com" => {
                    (fixer.to_string(), "jack/status/20", SocialPlatform::Twitter)
                }
                "instagram.com" => (
                    fixer.to_string(),
                    "reel/DbCP6xzRzdo/",
                    SocialPlatform::Instagram,
                ),
                "reddit.com" => (
                    fixer.to_string(),
                    "r/osugame/comments/1xm08ih",
                    SocialPlatform::Reddit,
                ),
                "facebook.com" => (
                    fixer.to_string(),
                    "share/p/1DL7Uac7xc/",
                    SocialPlatform::Facebook,
                ),
                "spotify.com" => (
                    format!("open.{fixer}"),
                    "track/11dFghVXANMlKmJXsNCbNl",
                    SocialPlatform::Spotify,
                ),
                _ => (format!("vm.{fixer}"), "ZN88Qw7ns/", SocialPlatform::TikTok),
            };
            let link = format!("https://{host}/{path}");

            let links = parse_links_with_alt_fixers(&link);
            assert_eq!(links.len(), 1, "{link}");
            assert_eq!(links[0].platform, platform, "{link}");
            assert_eq!(links[0].url.as_str(), link.replace(fixer, native), "{link}");
            assert!(parse_links(&link).is_empty(), "{link}");
        }
        assert!(parse_links_with_alt_fixers("https://notfxtwitter.com/jack/status/20").is_empty());
    }

    #[test]
    fn applies_each_platforms_saved_display() {
        let mut links =
            parse_links("https://x.com/jack/status/20 https://vm.tiktok.com/ZN88Qw7ns/");
        apply_displays(
            &mut links,
            &[
                ("twitter".to_owned(), "media-only".to_owned()),
                ("tiktok".to_owned(), "unknown".to_owned()),
            ],
        );

        assert_eq!(links[0].display, SocialDisplay::MediaOnly);
        assert_eq!(links[1].display, SocialDisplay::Full);
    }
}

use reqwest::Url;

mod bluesky;
mod facebook;
mod instagram;
mod spotify;
mod tiktok;
pub(crate) mod twitter;

pub(super) const ABEMBED_API: &str = "https://abembed.com/api/";

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
}

trait EmbedSource {
    fn handles(&self, url: &Url) -> bool;
    fn request(
        &self,
        url: &Url,
        twitter_options: &twitter::Options,
        spoiler: bool,
    ) -> Option<Request>;
    fn display_name(&self) -> &'static str;
    fn accent_color(&self) -> u32;

    fn media_url(&self, _post_url: &str, _kind: &str, _url: &str) -> Option<String> {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Source {
    Twitter(twitter::Twitter),
    Bluesky(bluesky::Bluesky),
    Instagram(instagram::Instagram),
    Facebook(facebook::Facebook),
    TikTok(tiktok::TikTok),
    Spotify(spotify::Spotify),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Request {
    FxEmbed { url: String, spoiler: bool },
    AbEmbed { url: String, spoiler: bool },
    Component { url: String, spoiler: bool },
}

#[derive(Debug, PartialEq)]
pub(crate) struct Link {
    pub(crate) source: Source,
    url: Url,
    spoiler: bool,
}

impl Link {
    pub(crate) fn request(self, twitter_options: &twitter::Options) -> Option<Request> {
        self.source
            .handler()
            .request(&self.url, twitter_options, self.spoiler)
    }
}

impl Source {
    pub(crate) fn platform(self) -> SocialPlatform {
        match self {
            Self::Twitter(_) => SocialPlatform::Twitter,
            Self::Bluesky(_) => SocialPlatform::Bluesky,
            Self::Instagram(_) => SocialPlatform::Instagram,
            Self::Facebook(_) => SocialPlatform::Facebook,
            Self::TikTok(_) => SocialPlatform::TikTok,
            Self::Spotify(_) => SocialPlatform::Spotify,
        }
    }

    fn recognize(url: &Url) -> Option<Self> {
        match () {
            _ if spotify::Spotify.handles(url) => Some(Self::Spotify(spotify::Spotify)),
            _ if twitter::Twitter.handles(url) => Some(Self::Twitter(twitter::Twitter)),
            _ if bluesky::Bluesky.handles(url) => Some(Self::Bluesky(bluesky::Bluesky)),
            _ if instagram::Instagram.handles(url) => Some(Self::Instagram(instagram::Instagram)),
            _ if facebook::Facebook.handles(url) => Some(Self::Facebook(facebook::Facebook)),
            _ if tiktok::TikTok.handles(url) => Some(Self::TikTok(tiktok::TikTok)),
            _ => None,
        }
    }

    fn handler(&self) -> &dyn EmbedSource {
        match self {
            Self::Twitter(source) => source,
            Self::Bluesky(source) => source,
            Self::Instagram(source) => source,
            Self::Facebook(source) => source,
            Self::TikTok(source) => source,
            Self::Spotify(source) => source,
        }
    }

    pub(crate) fn from_provider(provider: &str) -> Option<Self> {
        match provider {
            "twitter" => Some(Self::Twitter(twitter::Twitter)),
            "bluesky" => Some(Self::Bluesky(bluesky::Bluesky)),
            "instagram" => Some(Self::Instagram(instagram::Instagram)),
            "facebook" => Some(Self::Facebook(facebook::Facebook)),
            "tiktok" => Some(Self::TikTok(tiktok::TikTok)),
            "spotify" => Some(Self::Spotify(spotify::Spotify)),
            _ => None,
        }
    }

    pub(crate) fn display_name(&self) -> &'static str {
        self.handler().display_name()
    }

    pub(crate) fn accent_color(&self) -> u32 {
        self.handler().accent_color()
    }

    pub(crate) fn media_url(&self, post_url: &str, kind: &str, url: &str) -> Option<String> {
        self.handler().media_url(post_url, kind, url)
    }
}

pub(crate) fn parse_links(content: &str) -> Vec<Link> {
    content
        .split_whitespace()
        .filter_map(|word| {
            let (url, spoiler) = parse_url(word)?;
            Some(Link {
                source: Source::recognize(&url)?,
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
            .any(|platform| platform == link.source.platform().key())
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
    fn every_supported_platform_has_an_independent_toggle() {
        for platform in SocialPlatform::ALL {
            let source = Source::from_provider(platform.key()).unwrap();
            assert_eq!(source.platform(), platform);
            let mut links = vec![Link {
                source,
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
            links.iter().map(|link| link.source).collect::<Vec<_>>(),
            [
                Source::Twitter(twitter::Twitter),
                Source::Spotify(spotify::Spotify),
                Source::TikTok(tiktok::TikTok),
            ]
        );
        assert!(!links[0].spoiler);
        assert!(links[1].spoiler);
        assert!(!links[2].spoiler);
    }
}

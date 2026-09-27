use reqwest::Url;

use super::{EmbedSource, Request};

const API: &str = "https://api.fxbsky.app/2/status/";
const HOSTS: &[&str] = &["bsky.app", "www.bsky.app"];

const DISPLAY_NAME: &str = "Bluesky";
const ACCENT_COLOR: u32 = 0x1185fe;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Bluesky;

impl EmbedSource for Bluesky {
    fn handles(&self, url: &Url) -> bool {
        post_parts(url).is_some()
    }

    fn request(
        &self,
        url: &Url,
        _twitter_options: &super::twitter::Options,
        spoiler: bool,
    ) -> Option<Request> {
        let (handle, rkey) = post_parts(url)?;
        Some(Request::FxEmbed {
            url: format!("{API}{handle}/{rkey}"),
            spoiler,
        })
    }

    fn display_name(&self) -> &'static str {
        DISPLAY_NAME
    }

    fn accent_color(&self) -> u32 {
        ACCENT_COLOR
    }

    fn media_url(&self, post_url: &str, kind: &str, _media_url: &str) -> Option<String> {
        if kind != "gif" {
            return None;
        }
        let mut url = Url::parse(post_url).ok()?;
        url.set_host(Some("d.fxbsky.app")).ok()?;
        Some(url.into())
    }
}

fn post_parts(url: &Url) -> Option<(&str, &str)> {
    let host = url.host_str()?;
    let parts: Vec<_> = url.path_segments()?.collect();
    match parts.as_slice() {
        ["profile", handle, "post", rkey, ..]
            if HOSTS.contains(&host) && !handle.is_empty() && !rkey.is_empty() =>
        {
            Some((handle, rkey))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_bluesky_posts_and_builds_the_fxembed_request() {
        let url = Url::parse("https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l").unwrap();
        assert_eq!(
            Bluesky.request(&url, &Default::default(), false),
            Some(Request::FxEmbed {
                url: "https://api.fxbsky.app/2/status/bsky.app/3l6oveex3ii2l".to_owned(),
                spoiler: false,
            })
        );
        assert!(!Bluesky.handles(&Url::parse("https://bsky.app/profile/bsky.app").unwrap()));
        assert_eq!(
            Bluesky
                .media_url("https://bsky.app/profile/a/post/b", "gif", "ignored")
                .as_deref(),
            Some("https://d.fxbsky.app/profile/a/post/b")
        );
    }
}

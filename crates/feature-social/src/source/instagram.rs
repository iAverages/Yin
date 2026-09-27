use reqwest::Url;

use super::{ABEMBED_API, EmbedSource, Request};

const HOSTS: &[&str] = &["instagram.com", "www.instagram.com"];

const DISPLAY_NAME: &str = "Instagram";
const ACCENT_COLOR: u32 = 0xce0071;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Instagram;

impl EmbedSource for Instagram {
    fn handles(&self, url: &Url) -> bool {
        shortcode(url).is_some()
    }

    fn request(
        &self,
        url: &Url,
        _twitter_options: &super::twitter::Options,
        spoiler: bool,
    ) -> Option<Request> {
        Some(Request::AbEmbed {
            url: format!("{ABEMBED_API}instagram/p/{}", shortcode(url)?),
            spoiler,
        })
    }

    fn display_name(&self) -> &'static str {
        DISPLAY_NAME
    }

    fn accent_color(&self) -> u32 {
        ACCENT_COLOR
    }
}

fn shortcode(url: &Url) -> Option<&str> {
    let host = url.host_str()?;
    let parts: Vec<_> = url.path_segments()?.collect();
    match parts.as_slice() {
        ["p" | "reel" | "reels" | "tv", shortcode, ..]
            if HOSTS.contains(&host)
                && !shortcode.is_empty()
                && shortcode
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')) =>
        {
            Some(shortcode)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_instagram_post_kinds_and_rejects_lookalikes() {
        let url = Url::parse("https://www.instagram.com/reels/DbCP6xzRzdo/").unwrap();
        assert_eq!(
            Instagram.request(&url, &Default::default(), false),
            Some(Request::AbEmbed {
                url: "https://abembed.com/api/instagram/p/DbCP6xzRzdo".to_owned(),
                spoiler: false,
            })
        );
        assert!(
            !Instagram.handles(
                &Url::parse("https://www.instagram.com.example/reels/DbCP6xzRzdo/").unwrap()
            )
        );
    }
}

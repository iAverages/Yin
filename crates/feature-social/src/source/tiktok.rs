use reqwest::Url;

use super::{ABEMBED_API, EmbedSource, Request};

const HOSTS: &[&str] = &["tiktok.com", "www.tiktok.com", "m.tiktok.com"];
const SHORT_HOSTS: &[&str] = &["vt.tiktok.com", "vm.tiktok.com"];

const DISPLAY_NAME: &str = "TikTok";
const ACCENT_COLOR: u32 = 0x1d9bf0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TikTok;

impl EmbedSource for TikTok {
    fn handles(&self, url: &Url) -> bool {
        route(url).is_some()
    }

    fn request(
        &self,
        url: &Url,
        _twitter_options: &super::twitter::Options,
        spoiler: bool,
    ) -> Option<Request> {
        Some(Request::AbEmbed {
            url: format!("{ABEMBED_API}tiktok/{}", route(url)?),
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

fn route(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    let parts: Vec<_> = url.path_segments()?.collect();

    match parts.as_slice() {
        [username, kind @ ("video" | "photo"), id, ..]
            if HOSTS.contains(&host)
                && username.starts_with('@')
                && (5..=30).contains(&id.len())
                && id.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            Some(format!("{username}/{kind}/{id}"))
        }
        ["t", id, ..]
            if HOSTS.contains(&host)
                && !id.is_empty()
                && id.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
        {
            Some((*id).to_owned())
        }
        [shortcode] | [shortcode, ""]
            if url.scheme() == "https"
                && SHORT_HOSTS.contains(&host)
                && (4..=32).contains(&shortcode.len())
                && shortcode.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
        {
            Some((*shortcode).to_owned())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_long_and_short_tiktok_links() {
        for (url, route) in [
            (
                "https://www.tiktok.com/@kopilawak/video/7665179028352945426",
                "@kopilawak/video/7665179028352945426",
            ),
            ("https://www.tiktok.com/t/ZP8T6SD9F", "ZP8T6SD9F"),
            ("https://vt.tiktok.com/ZSVhvYhGN/", "ZSVhvYhGN"),
            ("https://vm.tiktok.com/ZN88Qw7ns/", "ZN88Qw7ns"),
        ] {
            assert_eq!(
                TikTok.request(&Url::parse(url).unwrap(), &Default::default(), false,),
                Some(Request::AbEmbed {
                    url: format!("https://abembed.com/api/tiktok/{route}"),
                    spoiler: false,
                })
            );
        }
        assert!(!TikTok.handles(&Url::parse("https://vt.tiktok.com.example/ZSVhvYhGN/").unwrap()));
    }
}

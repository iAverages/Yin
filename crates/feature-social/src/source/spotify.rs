use reqwest::Url;

use super::{EmbedSource, Request};

const HOST: &str = "open.spotify.com";
const EMBED_HOST: &str = "open.spqtify.com";

const DISPLAY_NAME: &str = "Spotify";
const ACCENT_COLOR: u32 = 0x1d9bf0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Spotify;

impl EmbedSource for Spotify {
    fn handles(&self, url: &Url) -> bool {
        parts(url).is_some()
    }

    fn request(
        &self,
        url: &Url,
        _twitter_options: &super::twitter::Options,
        spoiler: bool,
    ) -> Option<Request> {
        parts(url)?;
        let mut url = url.clone();
        url.set_host(Some(EMBED_HOST)).ok()?;
        Some(Request::Component {
            url: url.into(),
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

fn parts(url: &Url) -> Option<(&str, &str)> {
    let path: Vec<_> = url.path_segments()?.collect();
    match path.as_slice() {
        [kind @ ("track" | "episode" | "album" | "playlist"), id]
            if url.scheme() == "https"
                && url.host_str() == Some(HOST)
                && id.len() == 22
                && id.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
        {
            Some((kind, id))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_supported_spotify_links_and_preserves_the_query() {
        for kind in ["track", "episode", "album", "playlist"] {
            let url = Url::parse(&format!(
                "https://open.spotify.com/{kind}/11dFghVXANMlKmJXsNCbNl?si=abc"
            ))
            .unwrap();
            assert_eq!(
                Spotify.request(&url, &Default::default(), true),
                Some(Request::Component {
                    url: format!("https://open.spqtify.com/{kind}/11dFghVXANMlKmJXsNCbNl?si=abc"),
                    spoiler: true,
                })
            );
        }
        assert!(!Spotify.handles(
            &Url::parse("https://open.spotify.com/artist/0LyfQWJT6nXafLPZqxe9Of").unwrap()
        ));
        assert!(!Spotify.handles(
            &Url::parse("https://open.spotify.com.example/track/11dFghVXANMlKmJXsNCbNl").unwrap()
        ));
    }
}

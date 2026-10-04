use reqwest::Url;

use super::Request;

const HOST: &str = "open.spotify.com";
const EMBED_HOST: &str = "open.spqtify.com";

pub(super) fn handles(url: &Url) -> bool {
    parts(url).is_some()
}

pub(super) fn request(url: &Url, spoiler: bool) -> Option<Request> {
    parts(url)?;
    let mut url = url.clone();
    url.set_host(Some(EMBED_HOST)).ok()?;
    Some(Request::Component {
        url: url.into(),
        spoiler,
    })
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
                request(&url, true),
                Some(Request::Component {
                    url: format!("https://open.spqtify.com/{kind}/11dFghVXANMlKmJXsNCbNl?si=abc"),
                    spoiler: true,
                })
            );
        }
        assert!(!handles(
            &Url::parse("https://open.spotify.com/artist/0LyfQWJT6nXafLPZqxe9Of").unwrap()
        ));
        assert!(!handles(
            &Url::parse("https://open.spotify.com.example/track/11dFghVXANMlKmJXsNCbNl").unwrap()
        ));
    }
}

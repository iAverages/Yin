use reqwest::Url;

use super::{ABEMBED_API, Request};

const HOSTS: &[&str] = &["facebook.com", "www.facebook.com", "m.facebook.com"];

pub(super) fn handles(url: &Url) -> bool {
    route(url).is_some()
}

pub(super) fn request(url: &Url, spoiler: bool) -> Option<Request> {
    Some(Request::AbEmbed {
        url: format!("{ABEMBED_API}facebook/{}", route(url)?),
        spoiler,
    })
}

fn route(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    let parts: Vec<_> = url.path_segments()?.collect();
    match parts.as_slice() {
        ["share", kind @ ("p" | "r" | "v"), code, ..]
            if HOSTS.contains(&host)
                && !code.is_empty()
                && code.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
        {
            Some(format!("share/{kind}/{code}"))
        }
        ["share", code, ..]
            if HOSTS.contains(&host)
                && !code.is_empty()
                && code.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
        {
            Some(format!("share/{code}"))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_supported_facebook_share_routes() {
        for (url, route) in [
            (
                "https://www.facebook.com/share/p/1DL7Uac7xc/?mibextid=wwXIfr",
                "share/p/1DL7Uac7xc",
            ),
            (
                "https://www.facebook.com/share/r/1MbWc8LmKs/?mibextid=wwXIfr",
                "share/r/1MbWc8LmKs",
            ),
            (
                "https://www.facebook.com/share/1DTvc3LK3k/?mibextid=wwXIfr",
                "share/1DTvc3LK3k",
            ),
            (
                "https://www.facebook.com/share/p/1KHNpv7kw6/?mibextid=wwXIfr",
                "share/p/1KHNpv7kw6",
            ),
            (
                "https://www.facebook.com/share/r/1EudwBSGoX/?mibextid=wwXIfr",
                "share/r/1EudwBSGoX",
            ),
            (
                "https://www.facebook.com/share/v/1DJnCygSRY/",
                "share/v/1DJnCygSRY",
            ),
        ] {
            assert_eq!(
                request(&Url::parse(url).unwrap(), false),
                Some(Request::AbEmbed {
                    url: format!("https://abembed.com/api/facebook/{route}"),
                    spoiler: false,
                })
            );
        }
    }
}

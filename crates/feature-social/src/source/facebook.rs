use reqwest::Url;

use super::Request;

const HOSTS: &[&str] = &["facebook.com", "www.facebook.com", "m.facebook.com"];

pub(super) fn handles(url: &Url) -> bool {
    route(url).is_some()
}

pub(super) fn request(url: &Url, abembed: &str, spoiler: bool) -> Option<Request> {
    Some(Request::AbEmbed {
        url: format!("{abembed}api/facebook/{}", route(url)?),
        spoiler,
    })
}

fn route(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    let path = url.path().trim_matches('/');

    if !HOSTS.contains(&host) || path.is_empty() {
        return None;
    }

    Some(match url.query() {
        Some(query) => format!("{path}/?{query}"),
        None => path.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_supported_facebook_share_routes() {
        for (url, route) in [
            (
                "https://www.facebook.com/share/p/1DL7Uac7xc/?mibextid=wwXIfr",
                "share/p/1DL7Uac7xc/?mibextid=wwXIfr",
            ),
            (
                "https://www.facebook.com/share/r/1MbWc8LmKs/?mibextid=wwXIfr",
                "share/r/1MbWc8LmKs/?mibextid=wwXIfr",
            ),
            (
                "https://www.facebook.com/share/1DTvc3LK3k/?mibextid=wwXIfr",
                "share/1DTvc3LK3k/?mibextid=wwXIfr",
            ),
            (
                "https://www.facebook.com/share/p/1KHNpv7kw6/?mibextid=wwXIfr",
                "share/p/1KHNpv7kw6/?mibextid=wwXIfr",
            ),
            (
                "https://www.facebook.com/share/r/1EudwBSGoX/?mibextid=wwXIfr",
                "share/r/1EudwBSGoX/?mibextid=wwXIfr",
            ),
            (
                "https://www.facebook.com/share/v/1DJnCygSRY/",
                "share/v/1DJnCygSRY",
            ),
            (
                "https://www.facebook.com/reel/2297089164473090",
                "reel/2297089164473090",
            ),
            (
                "https://www.facebook.com/photo/?fbid=1563210505837197&set=a.631632168995040",
                "photo/?fbid=1563210505837197&set=a.631632168995040",
            ),
        ] {
            assert_eq!(
                request(&Url::parse(url).unwrap(), "https://abembed.com/", false),
                Some(Request::AbEmbed {
                    url: format!("https://abembed.com/api/facebook/{route}"),
                    spoiler: false,
                })
            );
        }
    }
}

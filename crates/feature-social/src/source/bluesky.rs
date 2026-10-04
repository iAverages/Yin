use reqwest::Url;

use super::Request;

const API: &str = "https://api.fxbsky.app/2/status/";
const HOSTS: &[&str] = &["bsky.app", "www.bsky.app"];

pub(super) fn handles(url: &Url) -> bool {
    post_parts(url).is_some()
}

pub(super) fn request(url: &Url, spoiler: bool) -> Option<Request> {
    let (handle, rkey) = post_parts(url)?;
    Some(Request::FxEmbed {
        url: format!("{API}{handle}/{rkey}"),
        spoiler,
    })
}

pub(super) fn media_url(post_url: &str, kind: &str) -> Option<String> {
    if kind != "gif" {
        return None;
    }
    let mut url = Url::parse(post_url).ok()?;
    url.set_host(Some("d.fxbsky.app")).ok()?;
    Some(url.into())
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
            request(&url, false),
            Some(Request::FxEmbed {
                url: "https://api.fxbsky.app/2/status/bsky.app/3l6oveex3ii2l".to_owned(),
                spoiler: false,
            })
        );
        assert!(!handles(
            &Url::parse("https://bsky.app/profile/bsky.app").unwrap()
        ));
        assert_eq!(
            media_url("https://bsky.app/profile/a/post/b", "gif").as_deref(),
            Some("https://d.fxbsky.app/profile/a/post/b")
        );
    }
}

use reqwest::Url;

use super::Request;

const HOSTS: &[&str] = &["instagram.com", "www.instagram.com"];

pub(super) fn handles(url: &Url) -> bool {
    shortcode(url).is_some()
}

pub(super) fn request(url: &Url, abembed: &str, spoiler: bool) -> Option<Request> {
    Some(Request::AbEmbed {
        url: format!("{abembed}api/instagram/p/{}", shortcode(url)?),
        spoiler,
    })
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
            request(&url, "https://abembed.com/", false),
            Some(Request::AbEmbed {
                url: "https://abembed.com/api/instagram/p/DbCP6xzRzdo".to_owned(),
                spoiler: false,
            })
        );
        assert!(!handles(
            &Url::parse("https://www.instagram.com.example/reels/DbCP6xzRzdo/").unwrap()
        ));
    }
}

use reqwest::Url;

use super::Request;

const HOSTS: &[&str] = &["tiktok.com", "www.tiktok.com", "m.tiktok.com"];
const SHORT_HOSTS: &[&str] = &["vt.tiktok.com", "vm.tiktok.com"];

pub(super) fn handles(url: &Url) -> bool {
    route(url).is_some()
}

pub(super) fn request(url: &Url, abembed: &str, spoiler: bool) -> Option<Request> {
    Some(Request::AbEmbed {
        url: format!("{abembed}api/tiktok/{}", route(url)?),
        spoiler,
    })
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
                request(&Url::parse(url).unwrap(), "https://abembed.com/", false),
                Some(Request::AbEmbed {
                    url: format!("https://abembed.com/api/tiktok/{route}"),
                    spoiler: false,
                })
            );
        }
        assert!(!handles(
            &Url::parse("https://vt.tiktok.com.example/ZSVhvYhGN/").unwrap()
        ));
    }
}

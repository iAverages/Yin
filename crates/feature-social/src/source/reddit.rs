use reqwest::Url;

use super::Request;

const HOSTS: &[&str] = &[
    "reddit.com",
    "www.reddit.com",
    "old.reddit.com",
    "new.reddit.com",
    "m.reddit.com",
];

pub(super) fn handles(url: &Url) -> bool {
    route(url).is_some()
}

pub(super) fn request(url: &Url, abembed: &str, spoiler: bool) -> Option<Request> {
    Some(Request::AbEmbed {
        url: format!("{abembed}api/reddit/{}", route(url)?),
        spoiler,
    })
}

fn route(url: &Url) -> Option<String> {
    if !HOSTS.contains(&url.host_str()?) {
        return None;
    }
    let parts: Vec<_> = url
        .path_segments()?
        .filter(|part| !part.is_empty())
        .collect();
    match parts.as_slice() {
        ["r", subreddit, "comments", id] | ["r", subreddit, "comments", id, _]
            if valid_subreddit(subreddit) && valid_id(id) =>
        {
            Some(format!("r/{subreddit}/comments/{id}"))
        }
        ["r", subreddit, "comments", id, _, comment]
            if valid_subreddit(subreddit) && valid_id(id) && valid_id(comment) =>
        {
            Some(format!("r/{subreddit}/comments/{id}/comment/{comment}"))
        }
        ["r", subreddit, "s", code]
            if valid_subreddit(subreddit)
                && (1..=32).contains(&code.len())
                && code.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
        {
            Some(format!("r/{subreddit}/s/{code}"))
        }
        _ => None,
    }
}

fn valid_subreddit(subreddit: &str) -> bool {
    (2..=21).contains(&subreddit.len())
        && subreddit
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_id(id: &str) -> bool {
    (1..=16).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_post_comment_and_share_links() {
        for (url, route) in [
            (
                "https://www.reddit.com/r/osugame/comments/1xm08ih",
                "r/osugame/comments/1xm08ih",
            ),
            (
                "https://old.reddit.com/r/osugame/comments/1xm08ih/a_post_title/?utm_source=share",
                "r/osugame/comments/1xm08ih",
            ),
            (
                "https://reddit.com/r/osugame/comments/1xm08ih/comment/19roziotq9zur/",
                "r/osugame/comments/1xm08ih/comment/19roziotq9zur",
            ),
            (
                "https://m.reddit.com/r/osugame/comments/1xm08ih/a_post_title/19roziotq9zur/?context=3",
                "r/osugame/comments/1xm08ih/comment/19roziotq9zur",
            ),
            (
                "https://www.reddit.com/r/osugame/s/AbCdEf123/?utm_source=share",
                "r/osugame/s/AbCdEf123",
            ),
        ] {
            assert_eq!(
                request(&Url::parse(url).unwrap(), "https://abembed.com/", false),
                Some(Request::AbEmbed {
                    url: format!("https://abembed.com/api/reddit/{route}"),
                    spoiler: false,
                })
            );
        }
    }

    #[test]
    fn rejects_lookalikes_and_links_the_api_cannot_route() {
        for url in [
            "https://www.reddit.com.example/r/osugame/comments/1xm08ih",
            "https://www.reddit.com/r/osugame",
            "https://www.reddit.com/r/osugame/s/not-a-code",
            "https://www.reddit.com/user/someone/comments/1xm08ih",
            "https://redd.it/1xm08ih",
            "https://www.reddit.com/r/a/comments/1xm08ih",
            "https://www.reddit.com/r/osugame/comments/1XM08IH",
            "https://www.reddit.com/r/osugame/comments/1xm08ih/a_post_title/NotAnId",
        ] {
            assert!(!handles(&Url::parse(url).unwrap()), "{url}");
        }
    }
}

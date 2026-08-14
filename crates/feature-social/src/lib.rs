use std::sync::LazyLock;
use std::time::Duration;

use bot_core::Error;
use bot_core::serenity::{
    self,
    builder::EditMessage,
    http::{LightMethod, Request, Route},
};
use reqwest::{Client, RequestBuilder, Url, header::USER_AGENT};
use serde::Deserialize;
use serde_json::{Value, json};

const TWITTER_API: &str = "https://api.fxtwitter.com/2/status/";
const BLUESKY_API: &str = "https://api.fxbsky.app/2/status/";
const TWITTER_HOSTS: &[&str] = &[
    "x.com",
    "www.x.com",
    "twitter.com",
    "www.twitter.com",
    "mobile.twitter.com",
];
const BLUESKY_HOSTS: &[&str] = &["bsky.app", "www.bsky.app"];
const APP_USER_AGENT: &str = concat!("yin/", env!("CARGO_PKG_VERSION"));
static HTTP: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("valid FxEmbed HTTP client")
});

pub async fn handle_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

    let Some(api_url) = api_url(&message.content) else {
        return Ok(());
    };

    let post = request(&api_url)
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?
        .status;

    let body = serde_json::to_vec(&create_payload(&post))?;
    ctx.http
        .request(
            Request::new(
                Route::ChannelMessages {
                    channel_id: message.channel_id,
                },
                LightMethod::Post,
            )
            .body(Some(body)),
        )
        .await?;
    message
        .channel_id
        .edit_message(ctx, message.id, EditMessage::new().suppress_embeds(true))
        .await?;
    Ok(())
}

fn request(url: &str) -> RequestBuilder {
    HTTP.get(url).header(USER_AGENT, APP_USER_AGENT)
}

fn api_url(content: &str) -> Option<String> {
    content.split_whitespace().find_map(|word| {
        let url = Url::parse(word.trim_matches(|c: char| "<>()[]{}\"',.!?".contains(c))).ok()?;
        let host = url.host_str()?;
        let parts: Vec<_> = url.path_segments()?.collect();

        match parts.as_slice() {
            [_, "status", id, ..]
                if TWITTER_HOSTS.contains(&host)
                    && (2..=20).contains(&id.len())
                    && id.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                Some(format!("{TWITTER_API}{id}"))
            }
            ["profile", handle, "post", rkey, ..]
                if BLUESKY_HOSTS.contains(&host) && !handle.is_empty() && !rkey.is_empty() =>
            {
                Some(format!("{BLUESKY_API}{handle}/{rkey}"))
            }
            _ => None,
        }
    })
}

fn create_payload(post: &Post) -> Value {
    let mut components = Vec::new();
    append_post(&mut components, post, false);

    if let Some(quote) = &post.quote {
        components.push(json!({"type": 14, "divider": true, "spacing": 1}));
        match quote {
            Quote::Post(quote) => append_post(&mut components, quote, true),
            Quote::Tombstone(tombstone) => components.push(json!({
                "type": 10,
                "content": format!(
                    "**Quoted post unavailable**\n{}",
                    tombstone.message.as_deref().unwrap_or(&tombstone.reason),
                ),
            })),
        }
    }

    json!({
        "flags": 1 << 15,
        "allowed_mentions": {"parse": []},
        "components": [{
            "type": 17,
            "accent_color": if post.provider == "bluesky" { 0x1185fe } else { 0x1d9bf0 },
            "components": components,
        }],
    })
}

fn append_post(components: &mut Vec<Value>, post: &Post, quoted: bool) {
    let header = json!({
        "type": 10,
        "content": format!(
            "{}**{} (@{})**\n{}",
            if quoted { "**Quoted post**\n" } else { "" },
            post.author.name,
            post.author.screen_name,
            truncate(&post.text, 3500),
        ),
    });
    components.push(match &post.author.avatar_url {
        Some(avatar_url) => json!({
            "type": 9,
            "components": [header],
            "accessory": {"type": 11, "media": {"url": avatar_url}},
        }),
        None => header,
    });

    if !post.media.all.is_empty() {
        components.push(json!({
            "type": 12,
            "items": post.media.all.iter().take(10).map(|media| json!({
                "media": {"url": media_url(post, media)},
            })).collect::<Vec<_>>(),
        }));
    }

    let provider = match post.provider.as_str() {
        "twitter" => "X / Twitter",
        "bluesky" => "Bluesky",
        provider => provider,
    };
    components.push(json!({
        "type": 10,
        "content": format!(
            "❤️ {}   🔁 {}   💬 {}\n[View on {}]({})",
            post.likes, post.reposts, post.replies, provider, post.url,
        ),
    }));
}

fn media_url(post: &Post, media: &MediaItem) -> String {
    if media.kind != "gif" {
        return media.url.clone();
    }

    let host = match post.provider.as_str() {
        "twitter" => "d.fxtwitter.com",
        "bluesky" => "d.fxbsky.app",
        _ => return media.url.clone(),
    };
    let Ok(mut url) = Url::parse(&post.url) else {
        return media.url.clone();
    };
    if url.set_host(Some(host)).is_err() {
        return media.url.clone();
    }
    url.into()
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_owned();
    }
    value.chars().take(limit - 1).collect::<String>() + "..."
}

#[derive(Deserialize)]
struct Response {
    status: Post,
}

#[derive(Deserialize)]
struct Post {
    url: String,
    text: String,
    likes: u64,
    reposts: u64,
    replies: u64,
    author: Author,
    #[serde(default)]
    media: Media,
    provider: String,
    quote: Option<Quote>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Quote {
    Post(Box<Post>),
    Tombstone(Tombstone),
}

#[derive(Deserialize)]
struct Tombstone {
    reason: String,
    message: Option<String>,
}

#[derive(Deserialize)]
struct Author {
    name: String,
    screen_name: String,
    avatar_url: Option<String>,
}

#[derive(Default, Deserialize)]
struct Media {
    #[serde(default)]
    all: Vec<MediaItem>,
}

#[derive(Deserialize)]
struct MediaItem {
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_supported_post_links() {
        assert_eq!(
            api_url("look <https://x.com/jack/status/20?s=20>"),
            Some("https://api.fxtwitter.com/2/status/20".to_owned())
        );
        assert_eq!(
            api_url("https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l"),
            Some("https://api.fxbsky.app/2/status/bsky.app/3l6oveex3ii2l".to_owned())
        );
    }

    #[test]
    fn ignores_non_post_and_lookalike_links() {
        assert_eq!(api_url("https://x.com/jack"), None);
        assert_eq!(api_url("https://x.com.example/jack/status/20"), None);
        assert_eq!(api_url("https://bsky.app/profile/bsky.app"), None);
    }

    #[test]
    fn decodes_shared_api_response_and_media() {
        let response: Response = serde_json::from_str(
            r#"{
                "status": {
                    "url": "https://x.com/user/status/123",
                    "text": "post text",
                    "likes": 4,
                    "reposts": 3,
                    "replies": 2,
                    "provider": "twitter",
                    "author": {
                        "name": "User",
                        "screen_name": "user",
                        "url": "https://x.com/user",
                        "avatar_url": null
                    },
                    "media": {"all": [
                        {"type": "photo", "url": "https://example.com/image.jpg"},
                        {"type": "gif", "url": "https://example.com/video.mp4", "format": "video/mp4"}
                    ]},
                    "quote": {
                        "url": "https://x.com/quoted/status/456",
                        "text": "quoted text",
                        "likes": 8,
                        "reposts": 7,
                        "replies": 6,
                        "provider": "twitter",
                        "author": {
                            "name": "Quoted User",
                            "screen_name": "quoted",
                            "avatar_url": null
                        },
                        "media": {"all": [
                            {"type": "photo", "url": "https://example.com/quoted.jpg"}
                        ]}
                    }
                }
            }"#,
        )
        .unwrap();

        assert_eq!(response.status.media.all.len(), 2);
        let message = create_payload(&response.status);
        assert!(message.get("content").is_none());
        assert_eq!(message["flags"], 1 << 15);
        assert_eq!(
            message["components"][0]["components"][1]["items"][1]["media"]["url"],
            "https://d.fxtwitter.com/user/status/123"
        );
        assert!(
            message["components"][0]["components"][4]["content"]
                .as_str()
                .unwrap()
                .contains("quoted text")
        );
        assert_eq!(
            message["components"][0]["components"][5]["items"][0]["media"]["url"],
            "https://example.com/quoted.jpg"
        );
    }

    #[test]
    fn truncates_on_character_boundaries() {
        assert_eq!(truncate("hello", 5), "hello");
        assert_eq!(truncate("ab😀cd", 4), "ab😀...");
    }

    #[test]
    fn identifies_the_bot_to_fxembed() {
        let request = request("https://api.fxtwitter.com/2/status/20")
            .build()
            .unwrap();

        assert_eq!(request.headers()[USER_AGENT], APP_USER_AGENT);
    }
}

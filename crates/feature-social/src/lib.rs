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
const ABEMBED_API: &str = "https://i.kirsi.dev/api/";
const TWITTER_HOSTS: &[&str] = &[
    "x.com",
    "www.x.com",
    "twitter.com",
    "www.twitter.com",
    "mobile.twitter.com",
];
const BLUESKY_HOSTS: &[&str] = &["bsky.app", "www.bsky.app"];
const INSTAGRAM_HOSTS: &[&str] = &["instagram.com", "www.instagram.com"];
const TIKTOK_HOSTS: &[&str] = &["tiktok.com", "www.tiktok.com", "m.tiktok.com"];
const TIKTOK_SHORT_HOSTS: &[&str] = &["vt.tiktok.com", "vm.tiktok.com"];
const DESCRIPTION_LIMIT: usize = 500;
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

    let Some(api_url) = resolve_api_url(&message.content).await? else {
        return Ok(());
    };

    if let Err(error) = send_embed(ctx, message, &api_url).await {
        let _ = message.react(ctx, '❌').await;
        return Err(error);
    }

    message
        .channel_id
        .edit_message(ctx, message.id, EditMessage::new().suppress_embeds(true))
        .await?;
    Ok(())
}

async fn send_embed(
    ctx: &serenity::Context,
    message: &serenity::Message,
    api_url: &str,
) -> Result<(), Error> {
    let response = request(api_url).send().await?.error_for_status()?;
    let post = if api_url.starts_with(ABEMBED_API) {
        response.json::<LinkFixedPost>().await?.into()
    } else {
        response.json::<Response>().await?.status
    };

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
    Ok(())
}

fn request(url: &str) -> RequestBuilder {
    HTTP.get(url).header(USER_AGENT, APP_USER_AGENT)
}

async fn resolve_api_url(content: &str) -> Result<Option<String>, reqwest::Error> {
    if let Some(api_url) = api_url(content) {
        return Ok(Some(api_url));
    }

    let Some(url) = content.split_whitespace().find_map(parse_tiktok_short_url) else {
        return Ok(None);
    };
    let response = HTTP
        .head(url)
        .header(USER_AGENT, APP_USER_AGENT)
        .send()
        .await?
        .error_for_status()?;
    Ok(api_url(response.url().as_str()))
}

fn parse_tiktok_short_url(word: &str) -> Option<Url> {
    let url = Url::parse(word.trim_matches(|c: char| "<>()[]{}\"',.!?".contains(c))).ok()?;
    (url.scheme() == "https" && TIKTOK_SHORT_HOSTS.contains(&url.host_str()?)).then_some(url)
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
            ["p" | "reel" | "reels" | "tv", shortcode, ..]
                if INSTAGRAM_HOSTS.contains(&host)
                    && !shortcode.is_empty()
                    && shortcode.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
                    }) =>
            {
                Some(format!("{ABEMBED_API}instagram/p/{shortcode}"))
            }
            [username, kind @ ("video" | "photo"), id, ..]
                if TIKTOK_HOSTS.contains(&host)
                    && username.starts_with('@')
                    && (5..=30).contains(&id.len())
                    && id.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                Some(format!("{ABEMBED_API}tiktok/{username}/{kind}/{id}"))
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
            "accent_color": match post.provider.as_str() {
                "bluesky" => 0x1185fe,
                "instagram" => 0xce0071,
                _ => 0x1d9bf0,
            },
            "components": components,
        }],
    })
}

fn append_post(components: &mut Vec<Value>, post: &Post, quoted: bool) {
    let author = post.author.as_ref();
    let text = if post.text.is_empty() {
        format!("{} post", provider_name(&post.provider))
    } else {
        truncate(&post.text, DESCRIPTION_LIMIT)
    };
    let header = json!({
        "type": 10,
        "content": match author {
            Some(author) => format!(
                "{}**{}**\n{}",
                if quoted { "**Quoted post**\n" } else { "" },
                author_name(author),
                text,
            ),
            None => format!("{}{}", if quoted { "**Quoted post**\n" } else { "" }, text),
        },
    });
    components.push(match author.and_then(|author| author.avatar_url.as_ref()) {
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

    let stats = [
        post.likes.map(|value| format!("❤️ {value}")),
        post.reposts.map(|value| format!("🔁 {value}")),
        post.replies.map(|value| format!("💬 {value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("   ");
    let provider = provider_name(&post.provider);
    let link = format!("[View on {provider}]({})", post.url);
    components.push(json!({
        "type": 10,
        "content": if stats.is_empty() { link } else { format!("{stats}\n{link}") },
    }));
}

fn author_name(author: &Author) -> String {
    if author.screen_name.is_empty() || author.name.contains(&format!("@{}", author.screen_name)) {
        author.name.clone()
    } else {
        format!("{} (@{})", author.name, author.screen_name)
    }
}

fn provider_name(provider: &str) -> &str {
    match provider {
        "twitter" => "X / Twitter",
        "bluesky" => "Bluesky",
        "instagram" => "Instagram",
        "tiktok" => "TikTok",
        provider => provider,
    }
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
    value.chars().take(limit - 3).collect::<String>() + "..."
}

#[derive(Deserialize)]
struct Response {
    status: Post,
}

#[derive(Deserialize)]
struct Post {
    url: String,
    text: String,
    likes: Option<u64>,
    reposts: Option<u64>,
    replies: Option<u64>,
    author: Option<Author>,
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

#[derive(Deserialize)]
struct LinkFixedPost {
    url: String,
    description: Option<String>,
    author: Option<LinkFixedAuthor>,
    stats: LinkFixedStats,
    media: Vec<MediaItem>,
    platform: String,
}

#[derive(Deserialize)]
struct LinkFixedAuthor {
    name: String,
    username: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct LinkFixedStats {
    likes: Option<u64>,
    reposts: Option<u64>,
    comments: Option<u64>,
}

impl From<LinkFixedPost> for Post {
    fn from(post: LinkFixedPost) -> Self {
        Self {
            url: post.url,
            text: post.description.unwrap_or_default(),
            likes: post.stats.likes,
            reposts: post.stats.reposts,
            replies: post.stats.comments,
            author: post.author.map(|author| Author {
                name: author.name,
                screen_name: author.username.unwrap_or_default(),
                avatar_url: author.avatar_url,
            }),
            media: Media { all: post.media },
            provider: post.platform,
            quote: None,
        }
    }
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
        assert_eq!(
            api_url("https://www.instagram.com/reels/DbCP6xzRzdo/"),
            Some("https://i.kirsi.dev/api/instagram/p/DbCP6xzRzdo".to_owned())
        );
        assert_eq!(
            api_url("https://www.tiktok.com/@kopilawak/video/7665179028352945426"),
            Some("https://i.kirsi.dev/api/tiktok/@kopilawak/video/7665179028352945426".to_owned())
        );
    }

    #[test]
    fn recognizes_tiktok_short_links() {
        assert_eq!(
            parse_tiktok_short_url("<https://vt.tiktok.com/ZSVhvYhGN/>"),
            Some(Url::parse("https://vt.tiktok.com/ZSVhvYhGN/").unwrap())
        );
        assert_eq!(
            parse_tiktok_short_url("https://vm.tiktok.com/ZN88Qw7ns/"),
            Some(Url::parse("https://vm.tiktok.com/ZN88Qw7ns/").unwrap())
        );
        assert_eq!(
            parse_tiktok_short_url("https://vt.tiktok.com.example/ZSVhvYhGN/"),
            None
        );
    }

    #[test]
    fn ignores_non_post_and_lookalike_links() {
        assert_eq!(api_url("https://x.com/jack"), None);
        assert_eq!(api_url("https://x.com.example/jack/status/20"), None);
        assert_eq!(api_url("https://bsky.app/profile/bsky.app"), None);
        assert_eq!(api_url("https://www.instagram.com/poster/"), None);
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
        assert_eq!(truncate("😀abcd", 4), "😀...");
    }

    #[test]
    fn decodes_link_fixed_response() {
        let post: Post = serde_json::from_str::<LinkFixedPost>(
            r#"{
                "platform": "instagram",
                "id": "DbCP6xzRzdo",
                "url": "https://www.instagram.com/p/DbCP6xzRzdo/",
                "description": "post text",
                "author": {
                    "name": "User (@user)",
                    "username": "user",
                    "url": "https://www.instagram.com/user/",
                    "avatar_url": null
                },
                "stats": {"likes": 4, "reposts": null, "comments": 2},
                "media": [{
                    "type": "image",
                    "url": "https://example.com/image.jpg",
                    "width": 100,
                    "height": 100
                }]
            }"#,
        )
        .unwrap()
        .into();

        let message = create_payload(&post);
        assert_eq!(message["components"][0]["accent_color"], 0xce0071);
        assert_eq!(
            message["components"][0]["components"][0]["content"],
            "**User (@user)**\npost text"
        );
        assert_eq!(
            message["components"][0]["components"][1]["items"][0]["media"]["url"],
            "https://example.com/image.jpg"
        );
        assert_eq!(
            message["components"][0]["components"][2]["content"],
            "❤️ 4   💬 2\n[View on Instagram](https://www.instagram.com/p/DbCP6xzRzdo/)"
        );
    }

    #[test]
    fn identifies_the_bot_to_fxembed() {
        let request = request("https://api.fxtwitter.com/2/status/20")
            .build()
            .unwrap();

        assert_eq!(request.headers()[USER_AGENT], APP_USER_AGENT);
    }
}

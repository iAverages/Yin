use std::sync::LazyLock;
use std::time::Duration;

use bot_core::Error;
use bot_core::serenity::{
    self,
    builder::EditMessage,
    http::{LightMethod, Request, Route},
};
use reqwest::{
    Client, RequestBuilder, Url,
    header::{CONTENT_TYPE, RANGE, USER_AGENT},
};
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
const FACEBOOK_HOSTS: &[&str] = &["facebook.com", "www.facebook.com", "m.facebook.com"];
const TIKTOK_HOSTS: &[&str] = &["tiktok.com", "www.tiktok.com", "m.tiktok.com"];
const TIKTOK_SHORT_HOSTS: &[&str] = &["vt.tiktok.com", "vm.tiktok.com"];
const DESCRIPTION_LIMIT: usize = 500;
const MEDIA_GALLERY_ITEM_LIMIT: usize = 10;
const EMBED_API_RETRIES: u32 = 3;
const EMBED_API_RETRY_DELAY: Duration = Duration::from_secs(1);
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

    let result: Result<(), Error> = if let Some(url) = spotify_embed_url(&message.content) {
        send_spotify_embed(ctx, message, &url).await
    } else {
        let Some(api_url) = resolve_api_url(&message.content).await? else {
            return Ok(());
        };
        send_embed(ctx, message, &api_url).await
    };

    if let Err(error) = result {
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
    let post = fetch_post(api_url).await?;
    send_payload(ctx, message, create_payload(&post)).await
}

async fn send_spotify_embed(
    ctx: &serenity::Context,
    message: &serenity::Message,
    embed_url: &str,
) -> Result<(), Error> {
    let mut spotify_url = Url::parse(embed_url)?;
    spotify_url
        .set_host(Some("open.spotify.com"))
        .expect("validated Spotify embed URL");
    let metadata = request("https://open.spotify.com/oembed")
        .query(&[("url", spotify_url.as_str())])
        .send()
        .await?
        .error_for_status()?
        .json::<SpotifyOEmbed>()
        .await?;

    let (image_url, video_url) =
        spqtify_asset_urls(embed_url).expect("validated Spotify embed URL");
    let mut page_url = Url::parse(embed_url)?;
    page_url.set_query(None);
    page_url.set_fragment(None);

    let image_response = request(image_url.as_str())
        .send()
        .await?
        .error_for_status()?;
    let accent_color = image_response
        .headers()
        .get("x-basecolor")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_hex_color)
        .ok_or_else(|| std::io::Error::other("spqtify response had no valid base color"))?;
    drop(image_response);

    let response = request(video_url.as_str())
        .header(RANGE, "bytes=0-0")
        .send()
        .await?
        .error_for_status()?;
    if !response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("video/"))
    {
        return Err(std::io::Error::other("spqtify response was not a video").into());
    }

    send_payload(
        ctx,
        message,
        create_spotify_payload(
            &metadata.title,
            page_url.as_str(),
            video_url.as_str(),
            accent_color,
        ),
    )
    .await
}

async fn send_payload(
    ctx: &serenity::Context,
    message: &serenity::Message,
    payload: Value,
) -> Result<(), Error> {
    let body = serde_json::to_vec(&payload)?;
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

async fn fetch_post(api_url: &str) -> Result<Post, reqwest::Error> {
    for retry in 0..=EMBED_API_RETRIES {
        let result = async {
            let response = request(api_url).send().await?.error_for_status()?;
            if api_url.starts_with(ABEMBED_API) {
                Ok(response.json::<LinkFixedPost>().await?.into())
            } else {
                Ok(response.json::<Response>().await?.status)
            }
        }
        .await;

        match result {
            Ok(post) => return Ok(post),
            Err(_) if retry < EMBED_API_RETRIES => {
                tokio::time::sleep(EMBED_API_RETRY_DELAY * 2u32.pow(retry)).await;
            }
            Err(error) => return Err(error),
        }
    }

    unreachable!()
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

fn spotify_embed_url(content: &str) -> Option<String> {
    content.split_whitespace().find_map(|word| {
        let mut url =
            Url::parse(word.trim_matches(|c: char| "<>()[]{}\"',.!?".contains(c))).ok()?;
        let parts: Vec<_> = url.path_segments()?.collect();
        match parts.as_slice() {
            ["track" | "episode" | "album" | "playlist", id]
                if url.scheme() == "https"
                    && url.host_str() == Some("open.spotify.com")
                    && id.len() == 22
                    && id.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
            {
                url.set_host(Some("open.spqtify.com")).ok()?;
                Some(url.into())
            }
            _ => None,
        }
    })
}

fn spqtify_asset_urls(embed_url: &str) -> Option<(Url, Url)> {
    let url = Url::parse(embed_url).ok()?;
    let (kind, id) = {
        let mut parts = url.path_segments()?;
        let kind = parts.next()?.to_owned();
        let id = parts.next()?.to_owned();
        if parts.next().is_some() {
            return None;
        }
        (kind, id)
    };
    let prefix = matches!(kind.as_str(), "album" | "playlist")
        .then_some(format!("{kind}/"))
        .unwrap_or_default();

    let mut image_url = url.clone();
    image_url.set_path(&format!("/api/generate/image/{prefix}{id}"));
    let mut video_url = url;
    video_url.set_path(&format!("/api/generate/video/{prefix}{id}.mp4"));
    Some((image_url, video_url))
}

fn parse_hex_color(value: &str) -> Option<u32> {
    let value = value.strip_prefix('#')?;
    (value.len() == 6)
        .then(|| u32::from_str_radix(value, 16).ok())
        .flatten()
}

fn create_spotify_payload(
    title: &str,
    page_url: &str,
    video_url: &str,
    accent_color: u32,
) -> Value {
    json!({
        "flags": 1 << 15,
        "allowed_mentions": {"parse": []},
        "components": [{
            "type": 17,
            "accent_color": accent_color,
            "components": [
                {"type": 10, "content": format!("**[{title}]({page_url})**")},
                {"type": 12, "items": [{"media": {"url": video_url}}]},
            ],
        }],
    })
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
            ["share", kind @ ("p" | "r" | "v"), code, ..]
                if FACEBOOK_HOSTS.contains(&host)
                    && !code.is_empty()
                    && code.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
            {
                Some(format!("{ABEMBED_API}facebook/share/{kind}/{code}"))
            }
            ["share", code, ..]
                if FACEBOOK_HOSTS.contains(&host)
                    && !code.is_empty()
                    && code.bytes().all(|byte| byte.is_ascii_alphanumeric()) =>
            {
                Some(format!("{ABEMBED_API}facebook/share/{code}"))
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
                "facebook" => 0x1877f2,
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

    for media in post.media.all.chunks(MEDIA_GALLERY_ITEM_LIMIT) {
        components.push(json!({
            "type": 12,
            "items": media.iter().map(|media| json!({
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
        "facebook" => "Facebook",
        "tiktok" => "TikTok",
        provider => provider,
    }
}

fn media_url(post: &Post, media: &MediaItem) -> String {
    if media.kind != "gif" {
        return media.url.clone();
    }

    if post.provider == "twitter" {
        let Ok(mut url) = Url::parse(&media.url) else {
            return media.url.clone();
        };
        if url.host_str() != Some("video.twimg.com") || !url.path().ends_with(".mp4") {
            return media.url.clone();
        }
        let path = url.path().trim_end_matches(".mp4").to_owned() + ".gif";
        if url.set_host(Some("gif.fxtwitter.com")).is_ok() {
            url.set_path(&path);
            return url.into();
        }
        return media.url.clone();
    }

    let host = match post.provider.as_str() {
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
struct SpotifyOEmbed {
    title: String,
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

    const MESSAGE_COMPONENT_LIMIT: usize = 40;

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
                api_url(url),
                Some(format!("https://i.kirsi.dev/api/facebook/{route}"))
            );
        }
    }

    #[test]
    fn rewrites_supported_spotify_links_for_spqtify_embeds() {
        for kind in ["track", "episode", "album", "playlist"] {
            assert_eq!(
                spotify_embed_url(&format!(
                    "listen <https://open.spotify.com/{kind}/11dFghVXANMlKmJXsNCbNl?si=abc>"
                )),
                Some(format!(
                    "https://open.spqtify.com/{kind}/11dFghVXANMlKmJXsNCbNl?si=abc"
                ))
            );
        }
        assert_eq!(
            spotify_embed_url("https://open.spotify.com/artist/0LyfQWJT6nXafLPZqxe9Of"),
            None
        );
        assert_eq!(
            spotify_embed_url("https://open.spotify.com.example/track/11dFghVXANMlKmJXsNCbNl"),
            None
        );

        let payload = create_spotify_payload(
            "Cut To The Feeling",
            "https://open.spqtify.com/track/11dFghVXANMlKmJXsNCbNl",
            "https://open.spqtify.com/api/generate/video/11dFghVXANMlKmJXsNCbNl.mp4?si=abc",
            0x81c8cf,
        );
        assert_eq!(
            payload["components"][0]["components"][0]["content"],
            "**[Cut To The Feeling](https://open.spqtify.com/track/11dFghVXANMlKmJXsNCbNl)**"
        );
        assert_eq!(
            payload["components"][0]["components"][1]["items"][0]["media"]["url"],
            "https://open.spqtify.com/api/generate/video/11dFghVXANMlKmJXsNCbNl.mp4?si=abc"
        );
        assert_eq!(parse_hex_color("#81c8cf"), Some(0x81c8cf));
        assert_eq!(parse_hex_color("81c8cf"), None);

        let (image_url, video_url) =
            spqtify_asset_urls("https://open.spqtify.com/album/11dFghVXANMlKmJXsNCbNl?track=2")
                .unwrap();
        assert_eq!(
            image_url.as_str(),
            "https://open.spqtify.com/api/generate/image/album/11dFghVXANMlKmJXsNCbNl?track=2"
        );
        assert_eq!(
            video_url.as_str(),
            "https://open.spqtify.com/api/generate/video/album/11dFghVXANMlKmJXsNCbNl.mp4?track=2"
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
                        {"type": "gif", "url": "https://video.twimg.com/tweet_video/animation.mp4", "format": "video/mp4"}
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
            "https://gif.fxtwitter.com/tweet_video/animation.gif"
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
    fn splits_more_than_ten_images_across_media_galleries() {
        let message = create_payload(&post_with_media(11));
        let components = message["components"][0]["components"].as_array().unwrap();
        assert_eq!(components[1]["items"].as_array().unwrap().len(), 10);
        assert_eq!(components[2]["items"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn supports_discords_maximum_number_of_images() {
        let gallery_count = MESSAGE_COMPONENT_LIMIT - 3;
        let message = create_payload(&post_with_media(gallery_count * MEDIA_GALLERY_ITEM_LIMIT));
        let components = message["components"][0]["components"].as_array().unwrap();

        assert_eq!(components.len() + 1, MESSAGE_COMPONENT_LIMIT);
        assert_eq!(
            components[1..=gallery_count]
                .iter()
                .map(|gallery| gallery["items"].as_array().unwrap().len())
                .sum::<usize>(),
            370
        );
    }

    fn post_with_media(count: usize) -> Post {
        Post {
            url: "https://www.instagram.com/p/post/".to_owned(),
            text: "post text".to_owned(),
            likes: None,
            reposts: None,
            replies: None,
            author: None,
            media: Media {
                all: (0..count)
                    .map(|index| MediaItem {
                        kind: "image".to_owned(),
                        url: format!("https://example.com/{index}.jpg"),
                    })
                    .collect(),
            },
            provider: "instagram".to_owned(),
            quote: None,
        }
    }

    #[test]
    fn identifies_the_bot_to_fxembed() {
        let request = request("https://api.fxtwitter.com/2/status/20")
            .build()
            .unwrap();

        assert_eq!(request.headers()[USER_AGENT], APP_USER_AGENT);
    }

    #[test]
    fn embed_api_retries_use_increasing_backoff() {
        assert_eq!(
            (0..EMBED_API_RETRIES)
                .map(|retry| EMBED_API_RETRY_DELAY * 2u32.pow(retry))
                .collect::<Vec<_>>(),
            [
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(4)
            ]
        );
    }
}

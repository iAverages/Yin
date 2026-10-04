use crate::SocialPlatform;
use serde::Deserialize;
use serde_json::{Value, json};

const DESCRIPTION_LIMIT: usize = 500;
const MEDIA_GALLERY_ITEM_LIMIT: usize = 10;

#[derive(Clone, Copy)]
pub(crate) enum Format {
    FxEmbed,
    AbEmbed,
}

pub(crate) async fn decode(
    response: reqwest::Response,
    format: Format,
) -> Result<Post, reqwest::Error> {
    match format {
        Format::FxEmbed => Ok(response.json::<Response>().await?.status),
        Format::AbEmbed => Ok(response.json::<AbEmbedPost>().await?.into()),
    }
}

pub(crate) fn component(post: &Post, spoiler: bool) -> Value {
    let mut components = Vec::new();
    append(&mut components, post, false);

    if let Some(quote) = &post.quote {
        components.push(json!({"type": 14, "divider": true, "spacing": 1}));
        match quote {
            Quote::Post(quote) => append(&mut components, quote, true),
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
        "type": 17,
        "spoiler": spoiler,
        "accent_color": SocialPlatform::from_provider(&post.provider)
            .map_or(0x1d9bf0, SocialPlatform::accent_color),
        "components": components,
    })
}

fn append(components: &mut Vec<Value>, post: &Post, quoted: bool) {
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

    if let Some(translation) = &post.translation {
        components.push(json!({
            "type": 10,
            "content": format!(
                "**Translation ({})**\n{}",
                translation.target_lang,
                truncate(&translation.text, DESCRIPTION_LIMIT),
            ),
        }));
        components.push(json!({"type": 14, "divider": true, "spacing": 1}));
    }

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
    SocialPlatform::from_provider(provider)
        .map(SocialPlatform::display_name)
        .unwrap_or(provider)
}

fn media_url(post: &Post, media: &MediaItem) -> String {
    SocialPlatform::from_provider(&post.provider)
        .and_then(|platform| platform.media_url(&post.url, &media.kind, &media.url))
        .unwrap_or_else(|| media.url.clone())
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
pub(crate) struct Post {
    url: String,
    text: String,
    likes: Option<u64>,
    reposts: Option<u64>,
    replies: Option<u64>,
    author: Option<Author>,
    #[serde(default)]
    media: Media,
    provider: String,
    translation: Option<Translation>,
    quote: Option<Quote>,
}

#[derive(Deserialize)]
struct Translation {
    text: String,
    target_lang: String,
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
struct AbEmbedPost {
    url: String,
    description: Option<String>,
    author: Option<AbEmbedAuthor>,
    stats: AbEmbedStats,
    media: Vec<MediaItem>,
    platform: String,
}

#[derive(Deserialize)]
struct AbEmbedAuthor {
    name: String,
    username: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct AbEmbedStats {
    likes: Option<u64>,
    reposts: Option<u64>,
    comments: Option<u64>,
}

impl From<AbEmbedPost> for Post {
    fn from(post: AbEmbedPost) -> Self {
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
            translation: None,
            quote: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_fxembed_response_and_media() {
        let response: Response = serde_json::from_str(
            r#"{
                "status": {
                    "url": "https://x.com/user/status/123",
                    "text": "post text",
                    "likes": 4,
                    "reposts": 3,
                    "replies": 2,
                    "provider": "twitter",
                    "translation": {
                        "text": "translated post text",
                        "source_lang": "ja",
                        "target_lang": "en"
                    },
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
        let message = component(&response.status, false);
        assert_eq!(
            message["components"][0]["content"],
            "**Translation (en)**\ntranslated post text"
        );
        assert_eq!(message["components"][1]["type"], 14);
        assert_eq!(
            message["components"][2]["content"],
            "**User (@user)**\npost text"
        );
        assert_eq!(
            message["components"][3]["items"][1]["media"]["url"],
            "https://gif.fxtwitter.com/tweet_video/animation.gif"
        );
        assert!(
            message["components"][6]["content"]
                .as_str()
                .unwrap()
                .contains("quoted text")
        );
        assert_eq!(
            message["components"][7]["items"][0]["media"]["url"],
            "https://example.com/quoted.jpg"
        );
    }

    #[test]
    fn truncates_on_character_boundaries() {
        assert_eq!(truncate("hello", 5), "hello");
        assert_eq!(truncate("😀abcd", 4), "😀...");
    }

    #[test]
    fn decodes_abembed_response() {
        let post: Post = serde_json::from_str::<AbEmbedPost>(
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

        let message = component(&post, false);
        assert_eq!(message["accent_color"], 0xce0071);
        assert_eq!(
            message["components"][0]["content"],
            "**User (@user)**\npost text"
        );
        assert_eq!(
            message["components"][1]["items"][0]["media"]["url"],
            "https://example.com/image.jpg"
        );
        assert_eq!(
            message["components"][2]["content"],
            "❤️ 4   💬 2\n[View on Instagram](https://www.instagram.com/p/DbCP6xzRzdo/)"
        );
    }

    #[test]
    fn splits_more_than_ten_images_across_media_galleries() {
        let message = component(&post_with_media(11), true);
        let components = message["components"].as_array().unwrap();
        assert_eq!(message["spoiler"], true);
        assert_eq!(components[1]["items"].as_array().unwrap().len(), 10);
        assert_eq!(components[2]["items"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn supports_discords_maximum_number_of_images() {
        let gallery_count = 40 - 3;
        let message = component(
            &post_with_media(gallery_count * MEDIA_GALLERY_ITEM_LIMIT),
            false,
        );
        let components = message["components"].as_array().unwrap();

        assert_eq!(components.len() + 1, 40);
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
            translation: None,
            quote: None,
        }
    }
}

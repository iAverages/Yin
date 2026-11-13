use std::collections::HashMap;

use bot_core::Error;
use database::{Database, DatabaseError, KvRepository};
use feature_flags::FeatureFlagScope;
use regex::Regex;
use reqwest::Url;
use serde_json::Value;

use super::SocialPlatform;

pub const URL_FLAG: &str = "abembed-url";
const URL: &str = "https://abembed.com/";
const SPEC_KEY: &str = "abembed-openapi";
const SPEC_TTL_SECONDS: u32 = 30 * 60;

struct Site {
    platform: SocialPlatform,
    hosts: &'static [&'static str],
    skip: &'static [&'static str],
}

const SITES: &[Site] = &[
    Site {
        platform: SocialPlatform::Facebook,
        hosts: &["facebook.com", "www.facebook.com", "m.facebook.com"],
        skip: &[],
    },
    Site {
        platform: SocialPlatform::Instagram,
        hosts: &["instagram.com", "www.instagram.com"],
        skip: &[],
    },
    Site {
        platform: SocialPlatform::TikTok,
        hosts: &["tiktok.com", "www.tiktok.com", "m.tiktok.com"],
        // short links are on vt. and vm.
        skip: &["/tiktok/{shortcode}"],
    },
    Site {
        platform: SocialPlatform::TikTok,
        hosts: &["vt.tiktok.com", "vm.tiktok.com"],
        skip: &[],
    },
    Site {
        platform: SocialPlatform::Reddit,
        hosts: &[
            "reddit.com",
            "www.reddit.com",
            "old.reddit.com",
            "new.reddit.com",
            "m.reddit.com",
        ],
        skip: &[],
    },
];

pub(crate) fn platform(url: &Url) -> Option<SocialPlatform> {
    site(url).map(|site| site.platform)
}

pub(crate) async fn spec(database: &Database) -> Option<Spec> {
    match load_spec(database).await {
        Ok(spec) => Some(spec),
        Err(error) => {
            tracing::warn!(error = %error, "failed to load the abembed openapi spec");
            None
        }
    }
}

pub(crate) async fn base_url(
    database: &Database,
    user_id: u64,
    guild_id: Option<u64>,
) -> Result<String, DatabaseError> {
    let keys = url_keys(user_id, guild_id);
    let urls = KvRepository::new(database).get_many(&keys).await?;

    Ok(most_specific(&keys, urls))
}

pub async fn set_base_url(
    database: &Database,
    scope: FeatureFlagScope,
    value: &str,
) -> Result<bool, DatabaseError> {
    let kv = KvRepository::new(database);
    let key = url_key(scope);

    if value == "reset" {
        kv.delete(&key).await?;
        return Ok(true);
    }

    let Some(url) = parse_base_url(value) else {
        return Ok(false);
    };

    kv.set(&key, &url, None).await?;

    Ok(true)
}

fn parse_base_url(value: &str) -> Option<String> {
    let url = Url::parse(value).ok()?;
    let host = url.host_str()?;

    (url.scheme() == "https"
        && url.port().is_none()
        && (host == "abembed.com" || host.ends_with(".abembed.com")))
    .then(|| format!("https://{host}/"))
}

fn url_key(scope: FeatureFlagScope) -> String {
    match scope {
        FeatureFlagScope::Global => URL_FLAG.to_owned(),
        FeatureFlagScope::User(user_id) => format!("{URL_FLAG}:user:{user_id}"),
        FeatureFlagScope::Guild(guild_id) => format!("{URL_FLAG}:guild:{guild_id}"),
        FeatureFlagScope::Member { user_id, guild_id } => {
            format!("{URL_FLAG}:member:{user_id}:{guild_id}")
        }
    }
}

fn url_keys(user_id: u64, guild_id: Option<u64>) -> Vec<String> {
    [
        guild_id.map(|guild_id| FeatureFlagScope::Member { user_id, guild_id }),
        Some(FeatureFlagScope::User(user_id)),
        guild_id.map(FeatureFlagScope::Guild),
        Some(FeatureFlagScope::Global),
    ]
    .into_iter()
    .flatten()
    .map(url_key)
    .collect()
}

fn most_specific(keys: &[String], urls: Vec<(String, String)>) -> String {
    keys.iter()
        .find_map(|key| urls.iter().find(|(found, _)| found == key))
        .map_or_else(|| URL.to_owned(), |(_, url)| url.clone())
}

fn site(url: &Url) -> Option<&'static Site> {
    let host = url.host_str()?;
    SITES.iter().find(|site| site.hosts.contains(&host))
}

async fn load_spec(database: &Database) -> Result<Spec, Error> {
    let kv = KvRepository::new(database);

    if let Some(spec) = kv.get(SPEC_KEY).await? {
        return Spec::parse(&spec);
    }

    let body = crate::request(&format!("{URL}openapi.json"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let spec = Spec::parse(&body)?;
    kv.set(SPEC_KEY, &body, Some(SPEC_TTL_SECONDS)).await?;

    Ok(spec)
}

pub(crate) struct Spec {
    embeds: Vec<Route>,
    posts: Vec<String>,
}

struct Route {
    path: String,
    segments: Vec<Segment>,
}

enum Segment {
    Literal(String),
    Param(String, Check),
}

#[derive(Clone)]
enum Check {
    Any,
    Pattern(Regex),
    OneOf(Vec<String>),
}

impl Spec {
    pub(crate) fn parse(spec: &str) -> Result<Self, Error> {
        let spec = &serde_json::from_str::<Value>(spec)?;
        let paths = spec["paths"]
            .as_object()
            .ok_or("abembed openapi spec has no paths")?;

        let mut embeds = Vec::new();
        let mut posts = Vec::new();

        for (path, item) in paths {
            let operation = &item["get"];
            match operation["responses"]["200"]["$ref"].as_str() {
                Some("#/components/responses/DiscordEmbed") => {
                    embeds.push(Route::new(path, &checks(spec, operation)?));
                }
                Some("#/components/responses/Post") => posts.push(path.clone()),
                _ => {}
            }
        }

        Ok(Self { embeds, posts })
    }

    pub(super) fn component_url(&self, url: &Url, abembed: &str) -> Option<String> {
        let site = site(url)?;
        let platform = site.platform.key();

        let mut link = vec![platform];
        link.extend(url.path_segments()?.filter(|segment| !segment.is_empty()));

        let params = self
            .embeds
            .iter()
            .filter(|route| !site.skip.contains(&route.path.as_str()))
            .find_map(|route| route.params(&link))?;

        let api = format!("/api/{platform}/");

        let (_, path) = self
            .posts
            .iter()
            .filter(|post| post.starts_with(&api))
            .filter_map(|post| fill(post, &params))
            .max_by_key(|(used, _)| *used)?;

        let query = url
            .query()
            .filter(|_| params.contains_key("path"))
            .map(|query| format!("?{query}"))
            .unwrap_or_default();

        Some(format!("{abembed}{}/component{query}", &path[1..]))
    }
}

impl Route {
    fn new(path: &str, checks: &HashMap<String, Check>) -> Self {
        Self {
            path: path.to_owned(),
            segments: path
                .split('/')
                .skip(1)
                .map(|segment| match param_name(segment) {
                    Some(name) => Segment::Param(
                        name.to_owned(),
                        checks.get(name).cloned().unwrap_or(Check::Any),
                    ),
                    None => Segment::Literal(segment.to_owned()),
                })
                .collect(),
        }
    }

    fn params(&self, link: &[&str]) -> Option<HashMap<&str, String>> {
        let mut params = HashMap::new();

        for (index, segment) in self.segments.iter().enumerate() {
            let value = *link.get(index)?;
            match segment {
                Segment::Literal(literal) if literal == value => {}

                Segment::Param(name, _) if name == "path" => {
                    params.insert(name.as_str(), link[index..].join("/"));
                    return Some(params);
                }

                Segment::Param(name, check) if check.allows(value) => {
                    params.insert(name.as_str(), value.to_owned());
                }
                _ => return None,
            }
        }

        (link.len() == self.segments.len()).then_some(params)
    }
}

impl Check {
    fn allows(&self, value: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Pattern(pattern) => pattern.is_match(value),
            Self::OneOf(values) => values.iter().any(|allowed| allowed == value),
        }
    }
}

fn checks(spec: &Value, operation: &Value) -> Result<HashMap<String, Check>, Error> {
    let mut checks = HashMap::new();
    for parameter in operation["parameters"].as_array().into_iter().flatten() {
        let parameter = parameter["$ref"]
            .as_str()
            .and_then(|reference| spec.pointer(reference.strip_prefix('#')?))
            .unwrap_or(parameter);

        let Some(name) = parameter["name"].as_str() else {
            continue;
        };

        let schema = &parameter["schema"];

        let check = if let Some(pattern) = schema["pattern"].as_str() {
            Check::Pattern(Regex::new(pattern)?)
        } else if let Some(values) = schema["enum"].as_array() {
            Check::OneOf(
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            )
        } else {
            Check::Any
        };

        checks.insert(name.to_owned(), check);
    }

    Ok(checks)
}

fn fill(template: &str, params: &HashMap<&str, String>) -> Option<(usize, String)> {
    let mut used = 0;
    let path = template
        .split('/')
        .map(|segment| match param_name(segment) {
            Some(name) => {
                used += 1;
                params.get(name).map(String::as_str)
            }
            None => Some(segment),
        })
        .collect::<Option<Vec<_>>>()?
        .join("/");

    Some((used, path))
}

fn param_name(segment: &str) -> Option<&str> {
    segment.strip_prefix('{')?.strip_suffix('}')
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn component_url(url: &str) -> Option<String> {
        let route = |response: &str, parameters: &[&str]| {
            json!({"get": {
                "parameters": parameters
                    .iter()
                    .map(|name| json!({"$ref": format!("#/components/parameters/{name}")}))
                    .collect::<Vec<_>>(),
                "responses": {"200": {"$ref": format!("#/components/responses/{response}")}},
            }})
        };
        let parameter = |name: &str, schema: Value| json!({"name": name, "schema": schema});
        let spec = json!({
            "paths": {
                "/facebook/{path}": {"get": {
                    "parameters": [{"name": "path", "schema": {"type": "string"}}],
                    "responses": {"200": {"$ref": "#/components/responses/DiscordEmbed"}},
                }},
                "/api/facebook/{path}": route("Post", &[]),
                "/instagram/{media_type}/{shortcode}": route("DiscordEmbed", &["MediaType", "Shortcode"]),
                "/instagram/{username}/{media_type}/{shortcode}": route("DiscordEmbed", &["MediaType", "Shortcode"]),
                "/api/instagram/{media_type}/{shortcode}": route("Post", &[]),
                "/tiktok/t/{shortcode}": route("DiscordEmbed", &["TikTokShortcode"]),
                "/tiktok/{shortcode}": route("DiscordEmbed", &["TikTokShortcode"]),
                "/tiktok/{username}/{post_type}/{id}": route("DiscordEmbed", &["PostType", "TikTokId"]),
                "/tiktok/{username}/{post_type}/{id}/avatar": {"get": {"responses": {"302": {}}}},
                "/api/tiktok/{shortcode}": route("Post", &[]),
                "/api/tiktok/{username}/{post_type}/{id}": route("Post", &[]),
                "/reddit/r/{subreddit}/comments/{id}": route("DiscordEmbed", &["Subreddit", "RedditId"]),
                "/reddit/r/{subreddit}/comments/{id}/comment/{comment}": route("DiscordEmbed", &["Subreddit", "RedditId", "Comment"]),
                "/reddit/r/{subreddit}/comments/{id}/{slug}": route("DiscordEmbed", &["Subreddit", "RedditId"]),
                "/reddit/r/{subreddit}/comments/{id}/{slug}/{comment}": route("DiscordEmbed", &["Subreddit", "RedditId", "Comment"]),
                "/reddit/r/{subreddit}/s/{code}": route("DiscordEmbed", &["Subreddit", "ShareCode"]),
                "/api/reddit/r/{subreddit}/comments/{id}": route("Post", &[]),
                "/api/reddit/r/{subreddit}/comments/{id}/comment/{comment}": route("Post", &[]),
                "/api/reddit/r/{subreddit}/comments/{id}/{slug}": route("Post", &[]),
                "/api/reddit/r/{subreddit}/comments/{id}/{slug}/{comment}": route("Post", &[]),
                "/api/reddit/r/{subreddit}/s/{code}": route("Post", &[]),
            },
            "components": {"parameters": {
                "MediaType": parameter("media_type", json!({"enum": ["p", "reel", "reels", "tv"]})),
                "Shortcode": parameter("shortcode", json!({"pattern": "^[A-Za-z0-9_-]+$"})),
                "TikTokShortcode": parameter("shortcode", json!({"pattern": "^[A-Za-z0-9]{4,32}$"})),
                "PostType": parameter("post_type", json!({"enum": ["video", "photo"]})),
                "TikTokId": parameter("id", json!({"pattern": "^[0-9]{5,30}$"})),
                "Subreddit": parameter("subreddit", json!({"pattern": "^[A-Za-z0-9_]{2,21}$"})),
                "RedditId": parameter("id", json!({"pattern": "^[a-z0-9]{1,16}$"})),
                "Comment": parameter("comment", json!({"pattern": "^[a-z0-9]{1,16}$"})),
                "ShareCode": parameter("code", json!({"pattern": "^[A-Za-z0-9]{4,32}$"})),
            }},
        });
        Spec::parse(&spec.to_string())
            .unwrap()
            .component_url(&Url::parse(url).unwrap(), "https://abembed.com/")
    }

    #[test]
    fn accepts_only_https_abembed_hosts_as_base_urls() {
        for (value, url) in [
            ("https://abembed.com", "https://abembed.com/"),
            (
                "https://staging.abembed.com/reddit",
                "https://staging.abembed.com/",
            ),
            (
                "https://PR-12-previews.abembed.com/?a=b",
                "https://pr-12-previews.abembed.com/",
            ),
        ] {
            assert_eq!(parse_base_url(value).as_deref(), Some(url), "{value}");
        }
        for value in [
            "http://abembed.com",
            "https://abembed.com:8443",
            "https://abembed.com.example",
            "https://notabembed.com",
            "https://abembed.com@example.com",
            "staging.abembed.com",
            "true",
        ] {
            assert_eq!(parse_base_url(value), None, "{value}");
        }
    }

    #[test]
    fn the_most_specific_base_url_wins() {
        let keys = url_keys(1, Some(2));
        let url = |key: &str| (key.to_owned(), format!("https://{key}"));

        assert_eq!(
            keys,
            [
                "abembed-url:member:1:2",
                "abembed-url:user:1",
                "abembed-url:guild:2",
                "abembed-url"
            ]
        );
        assert_eq!(url_keys(1, None), ["abembed-url:user:1", "abembed-url"]);
        assert_eq!(
            most_specific(&keys, vec![url("abembed-url"), url("abembed-url:guild:2")]),
            "https://abembed-url:guild:2"
        );
        assert_eq!(most_specific(&keys, vec![]), "https://abembed.com/");
    }

    #[test]
    fn routes_links_to_the_component_endpoint_of_their_spec_route() {
        for (url, route) in [
            (
                "https://www.reddit.com/r/osugame/comments/1xm08ih",
                "reddit/r/osugame/comments/1xm08ih/component",
            ),
            (
                "https://old.reddit.com/r/osugame/comments/1xm08ih/a_post_title/?utm_source=share",
                "reddit/r/osugame/comments/1xm08ih/a_post_title/component",
            ),
            (
                "https://reddit.com/r/osugame/comments/1xm08ih/comment/19roziotq9zur/",
                "reddit/r/osugame/comments/1xm08ih/comment/19roziotq9zur/component",
            ),
            (
                "https://m.reddit.com/r/osugame/comments/1xm08ih/a_post_title/19roziotq9zur/?context=3",
                "reddit/r/osugame/comments/1xm08ih/a_post_title/19roziotq9zur/component",
            ),
            (
                "https://www.reddit.com/r/osugame/s/AbCdEf123/?utm_source=share",
                "reddit/r/osugame/s/AbCdEf123/component",
            ),
            (
                "https://www.tiktok.com/@kopilawak/video/7665179028352945426",
                "tiktok/@kopilawak/video/7665179028352945426/component",
            ),
            (
                "https://www.tiktok.com/t/ZP8T6SD9F",
                "tiktok/ZP8T6SD9F/component",
            ),
            (
                "https://vt.tiktok.com/ZSVhvYhGN/",
                "tiktok/ZSVhvYhGN/component",
            ),
            (
                "https://vm.tiktok.com/ZN88Qw7ns/",
                "tiktok/ZN88Qw7ns/component",
            ),
            (
                "https://www.instagram.com/reels/DbCP6xzRzdo/",
                "instagram/reels/DbCP6xzRzdo/component",
            ),
            (
                "https://www.instagram.com/someone/p/DbCP6xzRzdo/?img_index=1",
                "instagram/p/DbCP6xzRzdo/component",
            ),
            (
                "https://www.facebook.com/share/p/1DL7Uac7xc/?mibextid=wwXIfr",
                "facebook/share/p/1DL7Uac7xc/component?mibextid=wwXIfr",
            ),
            (
                "https://www.facebook.com/share/v/1DJnCygSRY/",
                "facebook/share/v/1DJnCygSRY/component",
            ),
            (
                "https://www.facebook.com/reel/2297089164473090",
                "facebook/reel/2297089164473090/component",
            ),
            (
                "https://www.facebook.com/photo/?fbid=1563210505837197&set=a.631632168995040",
                "facebook/photo/component?fbid=1563210505837197&set=a.631632168995040",
            ),
        ] {
            assert_eq!(
                component_url(url),
                Some(format!("https://abembed.com/api/{route}")),
                "{url}"
            );
        }
    }

    #[test]
    fn rejects_lookalikes_and_links_the_spec_has_no_route_for() {
        for url in [
            "https://www.reddit.com.example/r/osugame/comments/1xm08ih",
            "https://www.reddit.com/r/osugame",
            "https://www.reddit.com/r/osugame/s/not-a-code",
            "https://www.reddit.com/user/someone/comments/1xm08ih",
            "https://redd.it/1xm08ih",
            "https://www.reddit.com/r/a/comments/1xm08ih",
            "https://www.reddit.com/r/osugame/comments/1XM08IH",
            "https://www.reddit.com/r/osugame/comments/1xm08ih/a_post_title/NotAnId",
            "https://vt.tiktok.com.example/ZSVhvYhGN/",
            "https://www.tiktok.com/discover",
            "https://www.tiktok.com/@kopilawak",
            "https://www.instagram.com.example/reels/DbCP6xzRzdo/",
            "https://www.instagram.com/stories/DbCP6xzRzdo/",
            "https://www.facebook.com/",
        ] {
            assert_eq!(component_url(url), None, "{url}");
        }
    }
}

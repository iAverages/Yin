use bot_core::response::{Embed, EmbedKind};
use bot_core::{BotState, Error, raw, time};
use feature_flags::FeatureFlagScope;
use serde_json::{Value, json};
use twilight_http::routing::Route;
use twilight_model::channel::Message;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;

const USAGE: &str = "Usage: `admin`, `admin component <json>`, or \
    `admin flag <global|user|guild|member> <key> <value> [user] [guild]`.";

#[derive(Debug, PartialEq)]
enum AdminCommand<'a> {
    Info,
    Component(&'a str),
    Flag {
        key: &'a str,
        value: &'a str,
        scope: FeatureFlagScope,
    },
}

pub async fn handle_message(
    data: &BotState,
    message: &Message,
    prefix: &str,
    command_count: usize,
) -> Result<(), Error> {
    if !data.owner_ids.contains(&message.author.id) {
        return Ok(());
    }
    let Some(bot_id) = data.cache.current_user().map(|user| user.id) else {
        return Ok(());
    };
    let Some(command) = parse(&message.content, prefix, bot_id) else {
        return Ok(());
    };

    let embed = match command {
        Err(usage) => Embed::new(EmbedKind::Error, usage),
        Ok(AdminCommand::Info) => Embed::new(EmbedKind::Info, "Admin")
            .field("Environment", data.environment.to_string(), true)
            .field(
                "Uptime",
                time::format_duration(data.started_at.elapsed()),
                true,
            )
            .field("Guild Count", data.cache.stats().guilds().to_string(), true)
            .field("Command Count", command_count.to_string(), true),

        Ok(AdminCommand::Component(component)) => match component_payload(component) {
            Ok(payload) => {
                raw::request(
                    &data.http,
                    &Route::CreateMessage {
                        channel_id: message.channel_id.get(),
                    },
                    Some(&payload),
                )
                .await?;
                return Ok(());
            }
            Err(error) => Embed::new(EmbedKind::Error, error),
        },
        Ok(AdminCommand::Flag { key, value, scope }) => set_flag(data, key, value, scope).await?,
    };
    data.http
        .create_message(message.channel_id)
        .embeds(&[embed.into()])
        .await?;
    Ok(())
}

fn parse<'a>(
    content: &'a str,
    prefix: &str,
    bot_id: Id<UserMarker>,
) -> Option<Result<AdminCommand<'a>, &'static str>> {
    let rest = [
        prefix.to_owned(),
        format!("<@{bot_id}>"),
        format!("<@!{bot_id}>"),
    ]
    .iter()
    .find_map(|prefix| content.strip_prefix(prefix.as_str()))?
    .trim_start();
    let rest = rest.strip_prefix("admin")?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let (subcommand, args) = rest
        .trim()
        .split_once(char::is_whitespace)
        .unwrap_or((rest.trim(), ""));
    let args = args.trim();

    Some(match subcommand {
        "" => Ok(AdminCommand::Info),
        "component" if !args.is_empty() => Ok(AdminCommand::Component(args)),
        "flag" => parse_flag(args).ok_or(USAGE),
        _ => Err(USAGE),
    })
}

fn parse_flag(args: &str) -> Option<AdminCommand<'_>> {
    let words: Vec<_> = args.split_whitespace().collect();
    let scope = match words.as_slice() {
        ["global", _, _] => FeatureFlagScope::Global,
        ["user", _, _, user] => FeatureFlagScope::User(parse_user(user)?),
        ["guild", _, _, guild] => FeatureFlagScope::Guild(parse_id(guild)?),
        ["member", _, _, user, guild] => FeatureFlagScope::Member {
            user_id: parse_user(user)?,
            guild_id: parse_id(guild)?,
        },
        _ => return None,
    };
    Some(AdminCommand::Flag {
        key: words[1],
        value: words[2],
        scope,
    })
}

fn parse_user(value: &str) -> Option<u64> {
    let id = value
        .strip_prefix("<@")
        .and_then(|value| value.strip_suffix('>'))
        .map_or(value, |id| id.trim_start_matches('!'));
    parse_id(id)
}

fn parse_id(value: &str) -> Option<u64> {
    value.parse().ok().filter(|id| *id != 0)
}

fn component_payload(component: &str) -> Result<Value, String> {
    match serde_json::from_str::<Value>(component) {
        Ok(component) if component.is_object() => Ok(json!({
            "flags": 1 << 15,
            "allowed_mentions": {"parse": []},
            "components": [component],
        })),
        Ok(_) => Err("Component JSON must be an object.".to_owned()),
        Err(error) => Err(format!("Invalid JSON: {error}")),
    }
}

fn scope_name(scope: FeatureFlagScope) -> String {
    match scope {
        FeatureFlagScope::Global => "All users in all guilds".to_owned(),
        FeatureFlagScope::User(user) => format!("User `{user}` in all guilds"),
        FeatureFlagScope::Guild(guild) => format!("All users in guild `{guild}`"),
        FeatureFlagScope::Member { user_id, guild_id } => {
            format!("User `{user_id}` in guild `{guild_id}`")
        }
    }
}

async fn set_flag(
    data: &BotState,
    key: &str,
    value: &str,
    scope: FeatureFlagScope,
) -> Result<Embed, Error> {
    if key == feature_social::ABEMBED_URL_FLAG {
        if !feature_social::set_abembed_url(&data.database, scope, value).await? {
            return Ok(Embed::new(
                EmbedKind::Error,
                "Use an `https://` abembed.com URL, or `reset` to remove the override.",
            ));
        }
    } else if let Err(error) = data.feature_flags.set_value(key, scope, value).await {
        tracing::error!(flag = %key, value, ?scope, error = %error, "failed to update feature flag");
        let message = match error {
            feature_flags::Error::FlagNotFound(_) => {
                "That PostHog feature flag does not exist.".to_owned()
            }
            feature_flags::Error::Management { status, .. } => format!(
                "PostHog could not apply the change ({status}). Check the flag state and bot API-key scopes."
            ),
            feature_flags::Error::Request(_) => "The bot could not reach PostHog.".to_owned(),
            feature_flags::Error::InvalidValue { expected, .. } => {
                format!("That value is invalid for this flag. Expected: {expected}.")
            }
            _ => "PostHog feature flags are not configured correctly.".to_owned(),
        };
        return Ok(Embed::new(EmbedKind::Error, message));
    }
    Ok(Embed::new(EmbedKind::Success, "Feature Flag Updated")
        .field("Flag", format!("`{key}`"), true)
        .field("Value", format!("`{value}`"), true)
        .field("Scope", scope_name(scope), false))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOT: Id<UserMarker> = Id::new(42);

    #[test]
    fn parses_prefix_and_mention_invocations() {
        assert_eq!(parse("!admin", "!", BOT), Some(Ok(AdminCommand::Info)));
        assert_eq!(parse("$ admin", "$", BOT), Some(Ok(AdminCommand::Info)));
        assert_eq!(parse("<@42> admin", "!", BOT), Some(Ok(AdminCommand::Info)));
        assert_eq!(parse("<@!42>admin", "!", BOT), Some(Ok(AdminCommand::Info)));
        assert_eq!(parse("<@43> admin", "!", BOT), None);
        assert_eq!(parse("!administrator", "!", BOT), None);
        assert_eq!(parse("admin", "!", BOT), None);
    }

    #[test]
    fn parses_flag_scopes() {
        assert_eq!(
            parse("!admin flag member beta true <@!7> 9", "!", BOT),
            Some(Ok(AdminCommand::Flag {
                key: "beta",
                value: "true",
                scope: FeatureFlagScope::Member {
                    user_id: 7,
                    guild_id: 9
                },
            }))
        );
        assert_eq!(
            parse("!admin flag global beta off", "!", BOT),
            Some(Ok(AdminCommand::Flag {
                key: "beta",
                value: "off",
                scope: FeatureFlagScope::Global,
            }))
        );
    }

    #[test]
    fn rejects_unknown_subcommands_and_missing_args() {
        for content in [
            "!admin nope",
            "!admin flag",
            "!admin flag user beta true",
            "!admin flag guild beta true abc",
            "!admin component",
        ] {
            assert_eq!(parse(content, "!", BOT), Some(Err(USAGE)), "{content}");
        }
    }

    #[test]
    fn component_takes_the_rest_of_the_message() {
        let content = "!admin component {\"type\": 10, \"content\": \"hi there\"}";
        let Some(Ok(AdminCommand::Component(json))) = parse(content, "!", BOT) else {
            panic!("expected component");
        };
        let payload = component_payload(json).unwrap();

        assert_eq!(payload["flags"], 1 << 15);
        assert_eq!(payload["allowed_mentions"]["parse"], json!([]));
        assert_eq!(payload["components"][0]["content"], "hi there");
        assert!(component_payload("[]").is_err());
    }
}

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::serenity::{
    self,
    http::{LightMethod, Request, Route},
};
use bot_core::time;
use bot_core::{Context, Error, poise};
use feature_flags::FeatureFlagScope;
use serde_json::{Value, json};

#[poise::command(prefix_command, owners_only, subcommands("flag", "component"))]
pub async fn admin(ctx: Context<'_>) -> Result<(), Error> {
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Admin")
            .field("Environment", ctx.data().environment.to_string(), true)
            .field(
                "Uptime",
                time::format_duration(ctx.data().started_at.elapsed()),
                true,
            )
            .field(
                "Guild Count",
                ctx.serenity_context().cache.guild_count().to_string(),
                true,
            )
            .field(
                "Command Count",
                ctx.framework().options().commands.len().to_string(),
                true,
            )
            .field("Database", "Connected", true),
    )
    .await
}

#[poise::command(prefix_command, owners_only)]
async fn component(
    ctx: Context<'_>,
    #[rest]
    #[description = "Component V2 JSON"]
    component: String,
) -> Result<(), Error> {
    let component = match serde_json::from_str::<Value>(&component) {
        Ok(component) if component.is_object() => component,
        Ok(_) => return response::error(ctx, "Component JSON must be an object.").await,
        Err(error) => return response::error(ctx, format!("Invalid JSON: {error}")).await,
    };
    let body = component_payload(component)?;
    ctx.serenity_context()
        .http
        .fire::<serenity::Message>(
            Request::new(
                Route::ChannelMessages {
                    channel_id: ctx.channel_id(),
                },
                LightMethod::Post,
            )
            .body(Some(body)),
        )
        .await?;
    Ok(())
}

fn component_payload(component: Value) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&json!({
        "flags": 1 << 15,
        "allowed_mentions": {"parse": []},
        "components": [component],
    }))
}

#[poise::command(
    prefix_command,
    owners_only,
    subcommands("global", "user", "guild", "member")
)]
async fn flag(ctx: Context<'_>) -> Result<(), Error> {
    response::info(ctx, "Use `global`, `user`, `guild`, or `member`.").await
}

#[poise::command(prefix_command, owners_only)]
async fn global(ctx: Context<'_>, key: String, value: String) -> Result<(), Error> {
    set_flag(
        ctx,
        key,
        value,
        FeatureFlagScope::Global,
        "All users in all guilds".to_owned(),
    )
    .await
}

#[poise::command(prefix_command, owners_only)]
async fn user(
    ctx: Context<'_>,
    key: String,
    value: String,
    user: serenity::UserId,
) -> Result<(), Error> {
    set_flag(
        ctx,
        key,
        value,
        FeatureFlagScope::User(user.get()),
        format!("User `{user}` in all guilds"),
    )
    .await
}

#[poise::command(prefix_command, owners_only)]
async fn guild(
    ctx: Context<'_>,
    key: String,
    value: String,
    guild: serenity::GuildId,
) -> Result<(), Error> {
    set_flag(
        ctx,
        key,
        value,
        FeatureFlagScope::Guild(guild.get()),
        format!("All users in guild `{guild}`"),
    )
    .await
}

#[poise::command(prefix_command, owners_only)]
async fn member(
    ctx: Context<'_>,
    key: String,
    value: String,
    user: serenity::UserId,
    guild: serenity::GuildId,
) -> Result<(), Error> {
    set_flag(
        ctx,
        key,
        value,
        FeatureFlagScope::Member {
            user_id: user.get(),
            guild_id: guild.get(),
        },
        format!("User `{user}` in guild `{guild}`"),
    )
    .await
}

async fn set_flag(
    ctx: Context<'_>,
    key: String,
    value: String,
    scope: FeatureFlagScope,
    scope_name: String,
) -> Result<(), Error> {
    if let Err(error) = ctx
        .data()
        .feature_flags
        .set_value(&key, scope, &value)
        .await
    {
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
        return response::error(ctx, message).await;
    }
    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Feature Flag Updated")
            .field("Flag", format!("`{key}`"), true)
            .field("Value", format!("`{value}`"), true)
            .field("Scope", scope_name, false),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_component_v2_json_in_a_message() {
        let body = component_payload(json!({"type": 10, "content": "hello"})).unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["flags"], 1 << 15);
        assert_eq!(payload["allowed_mentions"]["parse"], json!([]));
        assert_eq!(payload["components"][0]["content"], "hello");
    }
}

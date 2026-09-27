use bot_core::response::{self, Embed, EmbedKind};
use bot_core::time;
use bot_core::{Context, Error, poise};

#[poise::command(prefix_command, owners_only, subcommands("flag"))]
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
async fn flag(ctx: Context<'_>, key: String, enabled: bool) -> Result<(), Error> {
    if let Err(error) = ctx.data().feature_flags.set_enabled(&key, enabled).await {
        tracing::error!(flag = %key, enabled, error = %error, "failed to update feature flag");
        let message = match error {
            feature_flags::Error::FlagNotFound(_) => {
                "That PostHog feature flag does not exist.".to_owned()
            }
            feature_flags::Error::Management { status, .. } => format!(
                "PostHog could not apply the change ({status}). Check the flag state and bot API-key scopes."
            ),
            feature_flags::Error::Request(_) => "The bot could not reach PostHog.".to_owned(),
            _ => "PostHog feature flags are not configured correctly.".to_owned(),
        };
        return response::error(ctx, message).await;
    }
    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Feature Flag Updated")
            .field("Flag", format!("`{key}`"), true)
            .field("State", if enabled { "Enabled" } else { "Disabled" }, true)
            .description("The flag's PostHog targeting rules were left unchanged."),
    )
    .await
}

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::time;
use bot_core::{Context, Error, poise};

#[poise::command(prefix_command, owners_only)]
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

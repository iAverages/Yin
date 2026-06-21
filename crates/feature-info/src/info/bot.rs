use bot_core::response::{self, Embed, EmbedKind};
use bot_core::time;
use bot_core::{Context, Error, poise};

/// View information about the bot.
#[poise::command(
    slash_command,
    prefix_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn bot(ctx: Context<'_>) -> Result<(), Error> {
    let current_user = ctx.serenity_context().cache.current_user().clone();
    let uptime = ctx.data().started_at.elapsed();

    let mut embed = Embed::new(EmbedKind::Info, "Bot Info")
        .field("Version", env!("CARGO_PKG_VERSION"), true)
        .field("Environment", ctx.data().environment.to_string(), true)
        .field("Uptime", time::format_duration(uptime), true)
        .field(
            "Guild Count",
            ctx.serenity_context().cache.guild_count().to_string(),
            true,
        );

    if let Some(avatar_url) = current_user.avatar_url() {
        embed = embed.thumbnail(avatar_url);
    }

    response::send(ctx, embed).await
}

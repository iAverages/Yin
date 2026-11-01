use bot_core::Error;
use bot_core::cdn;
use bot_core::command::CommandContext;
use bot_core::response::{Embed, EmbedKind};
use bot_core::time;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(name = "bot", desc = "View information about the bot")]
pub struct BotCommand;

pub async fn run(ctx: &CommandContext) -> Result<(), Error> {
    let state = &ctx.state;
    let uptime = state.started_at.elapsed();

    let mut embed = Embed::new(EmbedKind::Info, "Bot Info")
        .field("Version", env!("CARGO_PKG_VERSION"), true)
        .field("Environment", state.environment.to_string(), true)
        .field("Uptime", time::format_duration(uptime), true)
        .field(
            "Guild Count",
            state.cache.stats().guilds().to_string(),
            true,
        );

    if let Some(avatar_url) = state
        .cache
        .current_user()
        .and_then(|user| cdn::avatar_url(user.id, user.avatar.as_ref()))
    {
        embed = embed.thumbnail(avatar_url);
    }

    ctx.send(embed).await
}

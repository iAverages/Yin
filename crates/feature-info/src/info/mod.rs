mod bot;
mod guild;
mod user;

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("guild", "user", "bot"),
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Info Commands")
            .description("Use one of the available info subcommands.")
            .field("Server", "`!info guild` or `/info guild`", false)
            .field("User", "`!info user` or `/info user`", false)
            .field("Bot", "`!info bot` or `/info bot`", false),
    )
    .await
}

pub use self::bot::bot;
pub use guild::guild;
pub use user::{user, user_context};

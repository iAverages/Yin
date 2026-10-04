mod bot;
mod guild;
mod user;

use bot_core::{Context, Error, poise};

#[poise::command(
    prefix_command,
    slash_command,
    subcommand_required,
    subcommands("guild", "user", "bot"),
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn info(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

pub use self::bot::bot;
pub use guild::guild;
pub use user::{user, user_context};

mod language;
mod prefix;
mod social;

use bot_core::{Context, Error, poise};

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommand_required,
    subcommands("prefix", "language", "ladder", "social"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn settings(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

pub use language::language;
pub use prefix::{DEFAULT_PREFIX, prefix};
pub use social::{social, social_platform_keys, validate_social_platform};

fn ladder() -> bot_core::Command {
    feature_moderation::ladder_command()
}

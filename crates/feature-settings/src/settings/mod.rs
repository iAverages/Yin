mod prefix;

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("prefix", "ladder"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn settings(ctx: Context<'_>) -> Result<(), Error> {
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Settings Commands")
            .description("Use one of the available settings subcommands.")
            .field(
                "View Prefix",
                "`!settings prefix view` or `/settings prefix view`",
                false,
            )
            .field(
                "Set Prefix",
                "`!settings prefix set <prefix>` or `/settings prefix set`",
                false,
            )
            .field(
                "Reset Prefix",
                "`!settings prefix reset` or `/settings prefix reset`",
                false,
            ),
    )
    .await
}

pub use prefix::prefix;

fn ladder() -> bot_core::Command {
    feature_moderation::ladder_command()
}

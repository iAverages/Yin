use std::time::Instant;

use bot_core::command::{CommandContext, CommandFuture};
use bot_core::response::{Embed, EmbedKind};
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "ping",
    desc = "Check that the bot is responding",
    contexts = "guild bot_dm private_channel",
    integration_types = "guild_install user_install"
)]
pub struct PingCommand;

pub fn run(ctx: &CommandContext) -> CommandFuture<'_> {
    Box::pin(async move {
        let started = Instant::now();
        ctx.send(Embed::new(EmbedKind::Info, "Pong")).await?;
        let round_trip = started.elapsed();

        let embed = Embed::new(EmbedKind::Info, "Pong")
            .field(
                "Round-trip latency",
                format!("{}ms", round_trip.as_millis()),
                true,
            )
            .into();
        ctx.state
            .http
            .interaction(ctx.interaction.application_id)
            .update_response(&ctx.interaction.token)
            .embeds(Some(&[embed]))
            .await?;
        Ok(())
    })
}

use std::time::Instant;

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};

#[poise::command(
    slash_command,
    prefix_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let started = Instant::now();
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Pong").field(
            "Round-trip latency",
            format!("{}ms", started.elapsed().as_millis()),
            true,
        ),
    )
    .await
}

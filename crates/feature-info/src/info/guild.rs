use bot_core::response::{self, Embed, EmbedKind};
use bot_core::time;
use bot_core::{Context, Error, poise};

/// View information about this server.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn guild(ctx: Context<'_>) -> Result<(), Error> {
    let Some((name, guild_id, owner_id, member_count, icon_url)) = ctx.guild().map(|guild| {
        (
            guild.name.clone(),
            guild.id,
            guild.owner_id,
            guild.member_count,
            guild.icon_url(),
        )
    }) else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };

    let mut embed = Embed::new(EmbedKind::Info, "Server Info")
        .field("Name", name, true)
        .field("Guild ID", guild_id.to_string(), true)
        .field(
            "Created",
            time::discord_timestamp(guild_id.created_at()),
            false,
        )
        .field("Owner ID", owner_id.to_string(), true)
        .field("Member Count", member_count.to_string(), true);

    if let Some(icon_url) = icon_url {
        embed = embed.thumbnail(icon_url);
    }

    response::send(ctx, embed).await
}

use bot_core::Error;
use bot_core::cdn;
use bot_core::command::CommandContext;
use bot_core::response::{Embed, EmbedKind};
use bot_core::time;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_util::snowflake::Snowflake;

#[derive(CommandModel, CreateCommand)]
#[command(name = "guild", desc = "View information about this server")]
pub struct GuildCommand;

pub async fn run(ctx: &CommandContext) -> Result<(), Error> {
    let Some((name, guild_id, owner_id, member_count, icon_url)) = ctx
        .guild_id()
        .and_then(|guild_id| ctx.state.cache.guild(guild_id))
        .map(|guild| {
            (
                guild.name().to_owned(),
                guild.id(),
                guild.owner_id(),
                guild.member_count(),
                cdn::guild_icon_url(guild.id(), guild.icon()),
            )
        })
    else {
        return ctx
            .error("This command can only be used in a server.")
            .await;
    };

    let mut embed = Embed::new(EmbedKind::Info, "Server Info")
        .field("Name", name, true)
        .field("Guild ID", guild_id.to_string(), true)
        .field(
            "Created",
            time::discord_timestamp(guild_id.timestamp() / 1000),
            false,
        )
        .field("Owner ID", owner_id.to_string(), true)
        .field(
            "Member Count",
            member_count.map_or_else(|| "Unknown".to_owned(), |count| count.to_string()),
            true,
        );

    if let Some(icon_url) = icon_url {
        embed = embed.thumbnail(icon_url);
    }

    ctx.send(embed).await
}

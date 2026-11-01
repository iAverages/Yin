use bot_core::Error;
use bot_core::command::CommandContext;
use bot_core::response::{Embed, EmbedKind};
use database::settings::CommandPrefix;
use database::{Database, GuildSettingsRepository};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;

pub const DEFAULT_PREFIX: &str = "!";

pub async fn active_prefix(
    database: &Database,
    guild_id: Option<Id<GuildMarker>>,
) -> Result<String, Error> {
    let Some(guild_id) = guild_id else {
        return Ok(DEFAULT_PREFIX.to_owned());
    };
    Ok(GuildSettingsRepository::new(database)
        .find_by_guild_id(guild_id.get())
        .await?
        .and_then(|settings| settings.command_prefix)
        .unwrap_or_else(|| DEFAULT_PREFIX.to_owned()))
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "prefix",
    desc = "View, set, or reset this server's command prefix."
)]
pub enum PrefixCommand {
    #[command(name = "view")]
    View(ViewPrefix),
    #[command(name = "set")]
    Set(SetPrefix),
    #[command(name = "reset")]
    Reset(ResetPrefix),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "view", desc = "View this server's command prefix.")]
pub struct ViewPrefix;

#[derive(CommandModel, CreateCommand)]
#[command(name = "set", desc = "Set this server's command prefix.")]
pub struct SetPrefix {
    /// New command prefix
    prefix: String,
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "reset",
    desc = "Reset this server's command prefix to the default."
)]
pub struct ResetPrefix;

pub async fn run(ctx: &CommandContext, command: PrefixCommand) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let repository = GuildSettingsRepository::new(&ctx.state.database);

    match command {
        PrefixCommand::View(_) => {
            let settings = repository.find_by_guild_id(guild_id.get()).await?;
            let configured_prefix = settings.and_then(|settings| settings.command_prefix);
            let active_prefix = configured_prefix.as_deref().unwrap_or(DEFAULT_PREFIX);
            ctx.send(Embed::new(EmbedKind::Info, "").field(
                "Active Prefix",
                format!("`{active_prefix}`"),
                true,
            ))
            .await
        }
        PrefixCommand::Set(SetPrefix { prefix }) => {
            let Some(prefix) = CommandPrefix::parse(&prefix) else {
                return ctx
                    .send(Embed::new(EmbedKind::Error, "Invalid Prefix").description(
                        "Prefixes must be non-empty, at most 16 characters, and contain no whitespace.",
                    ))
                    .await;
            };
            repository.upsert_prefix(guild_id.get(), &prefix).await?;
            ctx.send(Embed::new(EmbedKind::Success, "Prefix Updated").field(
                "Active Prefix",
                format!("`{prefix}`"),
                true,
            ))
            .await
        }
        PrefixCommand::Reset(_) => {
            repository.clear_prefix(guild_id.get()).await?;
            ctx.send(Embed::new(EmbedKind::Success, "").field(
                "Active Prefix",
                format!("`{DEFAULT_PREFIX}`"),
                true,
            ))
            .await
        }
    }
}

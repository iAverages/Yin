use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};
use database::GuildSettingsRepository;
use database::settings::CommandPrefix;

pub const DEFAULT_PREFIX: &str = "!";

/// View, set, or reset this server's command prefix.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommand_required,
    subcommands("view_prefix", "set_prefix", "reset_prefix"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn prefix(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View this server's command prefix.
#[poise::command(prefix_command, slash_command, rename = "view", guild_only)]
async fn view_prefix(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");

    let repository = GuildSettingsRepository::new(&ctx.data().database);
    let settings = repository.find_by_guild_id(guild_id.get()).await?;
    let configured_prefix = settings.and_then(|settings| settings.command_prefix);
    let active_prefix = configured_prefix.as_deref().unwrap_or(DEFAULT_PREFIX);

    let embed =
        Embed::new(EmbedKind::Info, "").field("Active Prefix", format!("`{active_prefix}`"), true);

    response::send(ctx, embed).await
}

/// Set this server's command prefix.
#[poise::command(prefix_command, slash_command, rename = "set", guild_only)]
async fn set_prefix(
    ctx: Context<'_>,
    #[description = "New command prefix"] prefix: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");

    let Some(prefix) = CommandPrefix::parse(&prefix) else {
        return response::send(
            ctx,
            Embed::new(EmbedKind::Error, "Invalid Prefix").description(
                "Prefixes must be non-empty, at most 16 characters, and contain no whitespace.",
            ),
        )
        .await;
    };

    let repository = GuildSettingsRepository::new(&ctx.data().database);
    repository.upsert_prefix(guild_id.get(), &prefix).await?;

    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Prefix Updated").field(
            "Active Prefix",
            format!("`{prefix}`"),
            true,
        ),
    )
    .await
}

/// Reset this server's command prefix to the default.
#[poise::command(prefix_command, slash_command, rename = "reset", guild_only)]
async fn reset_prefix(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");

    let repository = GuildSettingsRepository::new(&ctx.data().database);
    repository.clear_prefix(guild_id.get()).await?;

    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "").field(
            "Active Prefix",
            format!("`{DEFAULT_PREFIX}`"),
            true,
        ),
    )
    .await
}

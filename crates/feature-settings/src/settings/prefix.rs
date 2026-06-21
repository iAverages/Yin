use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};
use database::GuildSettingsRepository;

const DEFAULT_PREFIX: &str = "!";
const MAX_PREFIX_LEN: usize = 16;

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("view", "set", "reset"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn prefix(ctx: Context<'_>) -> Result<(), Error> {
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Prefix Settings")
            .description("Use one of the available prefix subcommands.")
            .field(
                "View",
                "`!settings prefix view` or `/settings prefix view`",
                false,
            )
            .field(
                "Set",
                "`!settings prefix set <prefix>` or `/settings prefix set`",
                false,
            )
            .field(
                "Reset",
                "`!settings prefix reset` or `/settings prefix reset`",
                false,
            ),
    )
    .await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    check = "bot_core::permissions::require_manage_guild",
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn view(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };

    let repository = GuildSettingsRepository::new(&ctx.data().database);
    let settings = repository.find_by_guild_id(guild_id.get()).await?;
    let configured_prefix = settings.and_then(|settings| settings.command_prefix);
    let active_prefix = configured_prefix.as_deref().unwrap_or(DEFAULT_PREFIX);

    let embed =
        Embed::new(EmbedKind::Info, "").field("Active Prefix", format!("`{active_prefix}`"), true);

    response::send(ctx, embed).await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "New command prefix"] prefix: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };

    let Some(prefix) = validate_prefix(&prefix) else {
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

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    check = "bot_core::permissions::require_manage_guild",
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };

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

fn validate_prefix(prefix: &str) -> Option<String> {
    let prefix = prefix.trim();

    if prefix.is_empty()
        || prefix.chars().count() > MAX_PREFIX_LEN
        || prefix.chars().any(char::is_whitespace)
    {
        return None;
    }

    Some(prefix.to_owned())
}

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};
use database::GuildSettingsRepository;
use feature_social::{normalize_translation_language, primary_translation_language};

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("set", "reset"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn language(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let default = guild_language(ctx);
    let configured = GuildSettingsRepository::new(&ctx.data().database)
        .find_by_guild_id(guild_id.get())
        .await?
        .and_then(|settings| settings.translation_language);
    let language = configured.as_deref().unwrap_or(&default);

    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "").field("Translation Language", language, true),
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
pub async fn set(
    ctx: Context<'_>,
    #[description = "ISO language code, such as en, ja, pt-BR, or zh-Hant"] language: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let Some(language) = normalize_translation_language(&language) else {
        return response::error(
            ctx,
            "Use a valid ISO language code, such as `en` or `pt-BR`.",
        )
        .await;
    };

    GuildSettingsRepository::new(&ctx.data().database)
        .upsert_translation_language(guild_id.get(), &language)
        .await?;

    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Translation Language Updated").field(
            "Translation Language",
            language,
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
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    GuildSettingsRepository::new(&ctx.data().database)
        .clear_translation_language(guild_id.get())
        .await?;
    let language = guild_language(ctx);

    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Translation Language Reset").field(
            "Translation Language",
            language,
            true,
        ),
    )
    .await
}

fn guild_language(ctx: Context<'_>) -> String {
    ctx.guild()
        .and_then(|guild| primary_translation_language(&guild.preferred_locale))
        .unwrap_or_else(|| "en".to_owned())
}

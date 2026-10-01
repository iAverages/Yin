use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};
use database::GuildSettingsRepository;
use database::settings::TranslationLanguage;
use feature_social::primary_translation_language;

/// View, set, or reset this server's translation language.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommand_required,
    subcommands("view_language", "set_language", "reset_language"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn language(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View this server's translation language.
#[poise::command(prefix_command, slash_command, rename = "view", guild_only)]
async fn view_language(ctx: Context<'_>) -> Result<(), Error> {
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

/// Set this server's translation language.
#[poise::command(prefix_command, slash_command, rename = "set", guild_only)]
async fn set_language(
    ctx: Context<'_>,
    #[description = "ISO language code, such as en, ja, pt-BR, or zh-Hant"] language: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let Some(language) = TranslationLanguage::parse(&language) else {
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
            language.as_str(),
            true,
        ),
    )
    .await
}

/// Reset this server's translation language to the default.
#[poise::command(prefix_command, slash_command, rename = "reset", guild_only)]
async fn reset_language(ctx: Context<'_>) -> Result<(), Error> {
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

use bot_core::Error;
use bot_core::command::CommandContext;
use bot_core::response::{Embed, EmbedKind};
use database::GuildSettingsRepository;
use database::settings::TranslationLanguage;
use feature_social::primary_translation_language;
use twilight_interactions::command::{CommandModel, CreateCommand};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "language",
    desc = "View, set, or reset this server's translation language."
)]
pub enum LanguageCommand {
    #[command(name = "view")]
    View(ViewLanguage),
    #[command(name = "set")]
    Set(SetLanguage),
    #[command(name = "reset")]
    Reset(ResetLanguage),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "view", desc = "View this server's translation language.")]
pub struct ViewLanguage;

#[derive(CommandModel, CreateCommand)]
#[command(name = "set", desc = "Set this server's translation language.")]
pub struct SetLanguage {
    /// ISO language code, such as en, ja, pt-BR, or zh-Hant
    language: String,
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "reset",
    desc = "Reset this server's translation language to the default."
)]
pub struct ResetLanguage;

pub async fn run(ctx: &CommandContext, command: LanguageCommand) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let repository = GuildSettingsRepository::new(&ctx.state.database);

    match command {
        LanguageCommand::View(_) => {
            let default = guild_language(ctx);
            let configured = repository
                .find_by_guild_id(guild_id.get())
                .await?
                .and_then(|settings| settings.translation_language);
            let language = configured.as_deref().unwrap_or(&default);
            ctx.send(Embed::new(EmbedKind::Info, "").field("Translation Language", language, true))
                .await
        }
        LanguageCommand::Set(SetLanguage { language }) => {
            let Some(language) = TranslationLanguage::parse(&language) else {
                return ctx
                    .error("Use a valid ISO language code, such as `en` or `pt-BR`.")
                    .await;
            };
            repository
                .upsert_translation_language(guild_id.get(), &language)
                .await?;
            ctx.send(
                Embed::new(EmbedKind::Success, "Translation Language Updated").field(
                    "Translation Language",
                    language.as_str(),
                    true,
                ),
            )
            .await
        }
        LanguageCommand::Reset(_) => {
            repository
                .clear_translation_language(guild_id.get())
                .await?;
            ctx.send(
                Embed::new(EmbedKind::Success, "Translation Language Reset").field(
                    "Translation Language",
                    guild_language(ctx),
                    true,
                ),
            )
            .await
        }
    }
}

fn guild_language(ctx: &CommandContext) -> String {
    ctx.interaction
        .guild_locale
        .as_deref()
        .and_then(primary_translation_language)
        .unwrap_or_else(|| "en".to_owned())
}

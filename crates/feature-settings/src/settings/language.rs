use bot_core::command::CommandContext;
use bot_core::response::{Embed, EmbedKind};
use bot_core::{BotState, Error};
use database::settings::TranslationLanguage;
use database::{GuildSettingsRepository, UserSocialEmbedsRepository};
use feature_social::{LANGUAGES, primary_translation_language};
use serde_json::{Value, json};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;

pub(super) const PAGE_ID: &str = "settings:language";
const SELECT_ID: &str = "settings:language:select";
const RESET_ID: &str = "settings:language:reset";
const MORE_PREFIX: &str = "more:";
const INVALID_LANGUAGE: &str = "Use a valid ISO language code, such as `en` or `pt-BR`.";
// Discord allows 25 options in a select menu; the last one shows the next batch.
const BATCH_SIZE: usize = 24;

/// Shows the batch holding the saved language unless another `batch` is asked for.
fn user_components(saved: Option<&str>, batch: Option<usize>) -> Vec<Value> {
    let current = saved.map_or_else(
        || "your Discord language".to_owned(),
        |language| format!("`{language}`"),
    );
    let batches = LANGUAGES.len().div_ceil(BATCH_SIZE);
    let batch = batch.unwrap_or_else(|| {
        LANGUAGES
            .iter()
            .position(|(_, code)| saved == Some(*code))
            .map_or(0, |index| index / BATCH_SIZE)
    }) % batches;
    let mut options: Vec<Value> = LANGUAGES
        .iter()
        .skip(batch * BATCH_SIZE)
        .take(BATCH_SIZE)
        .map(|(name, code)| json!({"label": name, "value": code, "default": saved == Some(*code)}))
        .collect();
    options.push(json!({
        "label": "More languages...",
        "value": format!("{MORE_PREFIX}{}", (batch + 1) % batches),
    }));

    vec![json!({
        "type": 17,
        "accent_color": super::ACCENT_COLOR,
        "components": [
            {
                "type": 10,
                "content": format!(
                    "## Translation language\nThe language `/embed` translates posts into when \
                     you don't pick one.\n\n**Current:** {current}"
                ),
            },
            {
                "type": 1,
                "components": [{
                    "type": 3,
                    "custom_id": SELECT_ID,
                    "placeholder": "Choose a language",
                    "options": options,
                }],
            },
            {
                "type": 1,
                "components": [
                    {"type": 2, "style": 2, "label": "Use my Discord language", "custom_id": RESET_ID},
                    {"type": 2, "style": 2, "label": "Back", "custom_id": super::HOME_ID},
                ],
            },
        ],
    })]
}

/// Returns the page to show, or `None` if the interaction is not for this page.
pub(super) async fn handle_interaction(
    data: &BotState,
    interaction: &Interaction,
    component: &MessageComponentInteractionData,
) -> Result<Option<Vec<Value>>, Error> {
    let Some(user_id) = interaction.author_id().map(|id| id.get()) else {
        return Ok(None);
    };
    let repository = UserSocialEmbedsRepository::new(&data.database);
    let custom_id = component.custom_id.as_str();
    let mut batch = None;
    if custom_id == RESET_ID {
        repository.set_translation_language(user_id, None).await?;
    } else if custom_id == SELECT_ID {
        let Some(value) = component.values.first() else {
            return Ok(None);
        };
        if let Some(next) = value.strip_prefix(MORE_PREFIX) {
            batch = next.parse().ok();
        } else {
            let Some(language) = TranslationLanguage::parse(value) else {
                return Ok(None);
            };
            repository
                .set_translation_language(user_id, Some(&language))
                .await?;
        }
    } else if custom_id != PAGE_ID {
        return Ok(None);
    }
    let saved = repository.translation_language(user_id).await?;
    Ok(Some(user_components(saved.as_deref(), batch)))
}

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
                return ctx.error(INVALID_LANGUAGE).await;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The menu's options, split into its languages and the batch its last entry leads to.
    fn menu(saved: Option<&str>, batch: Option<usize>) -> (Vec<Value>, usize) {
        let page = user_components(saved, batch).remove(0);
        let mut options = page["components"][1]["components"][0]["options"]
            .as_array()
            .unwrap()
            .clone();
        assert!(options.len() <= 25);
        let more = options.pop().unwrap();
        let next = more["value"].as_str().unwrap().strip_prefix(MORE_PREFIX);
        (options, next.unwrap().parse().unwrap())
    }

    #[test]
    fn one_menu_pages_through_every_language_and_wraps_around() {
        let mut codes = Vec::new();
        let mut batch = 0;
        loop {
            let (options, next) = menu(None, Some(batch));
            codes.extend(options.into_iter().map(|option| option["value"].clone()));
            batch = next;
            if batch == 0 {
                break;
            }
        }
        let all: Vec<_> = LANGUAGES.iter().map(|(_, code)| *code).collect();
        assert_eq!(codes, all);
    }

    #[test]
    fn user_page_opens_on_the_saved_language() {
        let (name, code) = *LANGUAGES.last().unwrap();
        let (options, _) = menu(Some(code), None);
        let selected: Vec<_> = options
            .iter()
            .filter(|option| option["default"] == true)
            .map(|option| option["label"].as_str().unwrap())
            .collect();
        assert_eq!(selected, [name]);

        let page = &user_components(None, None)[0]["components"];
        assert!(
            page[0]["content"]
                .as_str()
                .unwrap()
                .ends_with("your Discord language")
        );
        assert_eq!(page[2]["components"][0]["custom_id"], RESET_ID);
    }
}

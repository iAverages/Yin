use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise, serenity};

use crate::client::{BASE_URL, CodexClient, asset_url, item_url};
use crate::model::{
    BlackboardEntry, BlackboardValue, CatalogEntry, CodexType, EquipmentSet, OperatorPayload,
    Skill, Weapon,
};

const MAX_CHOICES: usize = 25;

#[poise::command(
    slash_command,
    subcommands("codex"),
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn endfield(ctx: Context<'_>) -> Result<(), Error> {
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Arknights: Endfield Tools")
            .description("Maps, Codex data, and Dijiang base-skill planning from akendfield.tools.")
            .field(
                "Interactive Map",
                format!("[Valley IV]({BASE_URL}/map/valley-iv) | [Wuling]({BASE_URL}/map/wuling)"),
                false,
            )
            .field(
                "Codex",
                format!("[Operators, weapons, and equipment]({BASE_URL}/codex)"),
                false,
            )
            .field(
                "Dijiang",
                format!("[Base skills]({BASE_URL}/dijiang/manufacturing?focus=overall&tier=max)"),
                false,
            ),
    )
    .await
}

/// Look up an operator, weapon, or equipment set.
#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn codex(
    ctx: Context<'_>,
    #[description = "Codex category"]
    #[rename = "type"]
    kind: CodexType,
    #[description = "Select a Codex entry"]
    #[autocomplete = "autocomplete_codex"]
    search: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let client = CodexClient::new();
    let manifest = client.manifest().await?;

    let embed = match kind {
        CodexType::Operator => {
            let entries = client.operators(&manifest).await?;
            let Some(entry) = find_entry(&entries, &search) else {
                return invalid_selection(ctx).await;
            };
            let slug = entry_slug(entry, &entries);
            match client.operator(&manifest, entry.id()).await? {
                Some(operator) => operator_embed(operator, &slug),
                None => return invalid_selection(ctx).await,
            }
        }
        CodexType::Weapon => {
            let entries = client.weapons(&manifest).await?;
            let Some(entry) = find_entry(&entries, &search) else {
                return invalid_selection(ctx).await;
            };
            let slug = entry_slug(entry, &entries);
            match client.weapon(&manifest, entry.id()).await? {
                Some(weapon) => weapon_embed(weapon, &slug),
                None => return invalid_selection(ctx).await,
            }
        }
        CodexType::Equipment => {
            let equipment = client.equipment(&manifest).await?;
            let Some(entry) = find_entry(&equipment, &search) else {
                return invalid_selection(ctx).await;
            };
            let slug = entry_slug(entry, &equipment);
            let id = entry.id().to_owned();
            equipment_embed(
                equipment
                    .into_iter()
                    .find(|entry| entry.id == id)
                    .expect("entry was found"),
                &slug,
            )
        }
    };

    response::send(ctx, embed).await
}

async fn autocomplete_codex(ctx: Context<'_>, partial: &str) -> Vec<serenity::AutocompleteChoice> {
    let Some(kind) = selected_type(ctx) else {
        return Vec::new();
    };
    let client = CodexClient::new();
    let Ok(manifest) = client.manifest().await else {
        return Vec::new();
    };

    match kind {
        CodexType::Operator => client
            .operators(&manifest)
            .await
            .map(|entries| choices(&entries, partial))
            .unwrap_or_default(),
        CodexType::Weapon => client
            .weapons(&manifest)
            .await
            .map(|entries| choices(&entries, partial))
            .unwrap_or_default(),
        CodexType::Equipment => client
            .equipment(&manifest)
            .await
            .map(|entries| choices(&entries, partial))
            .unwrap_or_default(),
    }
}

fn selected_type(ctx: Context<'_>) -> Option<CodexType> {
    let poise::Context::Application(ctx) = ctx else {
        return None;
    };
    let value = ctx
        .args
        .iter()
        .find(|argument| argument.name == "type")
        .and_then(|argument| match argument.value {
            serenity::ResolvedValue::Integer(value) => Some(value),
            _ => None,
        })?;

    match value {
        0 => Some(CodexType::Operator),
        1 => Some(CodexType::Weapon),
        2 => Some(CodexType::Equipment),
        _ => None,
    }
}

fn choices<T: CatalogEntry>(entries: &[T], partial: &str) -> Vec<serenity::AutocompleteChoice> {
    let partial = partial.trim().to_lowercase();
    entries
        .iter()
        .filter(|entry| {
            partial.is_empty()
                || entry.name().to_lowercase().contains(&partial)
                || entry.id().to_lowercase().contains(&partial)
        })
        .take(MAX_CHOICES)
        .map(|entry| serenity::AutocompleteChoice::new(entry.name(), entry.id()))
        .collect()
}

fn find_entry<'a, T: CatalogEntry>(entries: &'a [T], search: &str) -> Option<&'a T> {
    let search = search.trim();
    if let Some(entry) = entries.iter().find(|entry| entry.id() == search) {
        return Some(entry);
    }

    let mut matches = entries
        .iter()
        .filter(|entry| entry.name().eq_ignore_ascii_case(search));
    let entry = matches.next()?;
    matches.next().is_none().then_some(entry)
}

fn operator_embed(payload: OperatorPayload, slug: &str) -> Embed {
    let operator = payload.operator;
    let description = payload
        .details
        .as_ref()
        .and_then(|details| details.description.as_deref())
        .map(|description| summarize(description, 350));
    let element = payload
        .details
        .and_then(|details| details.element)
        .map(|element| element.name)
        .unwrap_or_else(|| "Unknown".to_owned());
    let voice_actors = operator
        .voice_actors
        .iter()
        .map(|actor| format!("{}: {}", voice_language(&actor.language), actor.name))
        .collect::<Vec<_>>()
        .join("\n");
    let mut embed = Embed::new(EmbedKind::Info, operator.name)
        .description(match description {
            Some(description) => format!(
                "{description}\n\n[View full operator details]({})",
                item_url(CodexType::Operator, slug)
            ),
            None => format!(
                "[View full operator details]({})",
                item_url(CodexType::Operator, slug)
            ),
        })
        .field("Rarity", rarity(operator.rarity), true)
        .field("Class", friendly_label(&operator.profession), true)
        .field("Element", element, true)
        .field("Weapon", operator.weapon_type, true)
        .field("Max Level", operator.max_level.to_string(), true)
        .field("Combat Skills", skill_names(&operator.combat_skills), false);
    if !voice_actors.is_empty() {
        embed = embed.field("Voice Actors", voice_actors, false);
    }
    with_image(embed, operator.portrait_url)
}

fn weapon_embed(weapon: Weapon, slug: &str) -> Embed {
    let description = summarize(&weapon.description, 500);
    with_thumbnail(
        Embed::new(EmbedKind::Info, weapon.name)
            .description(format!(
                "{description}\n\n[View full weapon details]({})",
                item_url(CodexType::Weapon, slug)
            ))
            .field("Rarity", rarity(weapon.rarity), true)
            .field("Type", weapon.weapon_type, true)
            .field("Max Level", weapon.max_level.to_string(), true)
            .field("Effects & Skills", skill_names(&weapon.skills), false),
        weapon.image_url,
    )
}

fn equipment_embed(equipment: EquipmentSet, slug: &str) -> Embed {
    let pieces = equipment
        .pieces
        .iter()
        .map(|piece| {
            format!(
                "{} - {} / {}",
                piece.name,
                rarity(piece.rarity),
                friendly_label(&piece.equipment_type)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let bonuses = equipment
        .set_bonuses
        .iter()
        .map(|bonus| {
            let description = summarize(
                &format_game_text(&bonus.description, &bonus.blackboard),
                300,
            );
            format!("**{}-piece: {}**\n{description}", bonus.pieces, bonus.name)
        })
        .collect::<Vec<_>>()
        .join("\n");
    with_thumbnail(
        Embed::new(EmbedKind::Info, equipment.name)
            .description(format!(
                "[View full equipment details]({})",
                item_url(CodexType::Equipment, slug)
            ))
            .field("Pieces", pieces, false)
            .field("Set Bonuses", bonuses, false),
        equipment.image_url,
    )
}

fn with_thumbnail(embed: Embed, image_url: Option<String>) -> Embed {
    match image_url {
        Some(image_url) => embed.thumbnail(asset_url(&image_url)),
        None => embed,
    }
}

fn with_image(embed: Embed, image_url: Option<String>) -> Embed {
    match image_url {
        Some(image_url) => embed.image(asset_url(&image_url)),
        None => embed,
    }
}

fn entry_slug<T: CatalogEntry>(entry: &T, entries: &[T]) -> String {
    let base = friendly_name_id(entry.name());
    let mut duplicates = entries
        .iter()
        .filter(|candidate| friendly_name_id(candidate.name()) == base)
        .collect::<Vec<_>>();
    duplicates.sort_by(|left, right| left.id().cmp(right.id()));
    if duplicates.len() < 2 {
        base
    } else {
        let index = duplicates
            .iter()
            .position(|candidate| candidate.id() == entry.id())
            .unwrap_or(0);
        format!("{base}-{}", index + 1)
    }
}

fn friendly_name_id(name: &str) -> String {
    let mut slug = String::new();
    for character in name.chars().flat_map(char::to_lowercase) {
        if character == '\'' || character == '\u{2019}' {
            continue;
        }
        if character.is_ascii_alphanumeric() {
            slug.push(character);
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_owned()
}

fn rarity(rarity: Option<u8>) -> String {
    rarity.map_or_else(|| "Unknown".to_owned(), |rarity| format!("{rarity} star"))
}

fn skill_names(skills: &[Skill]) -> String {
    if skills.is_empty() {
        "None".to_owned()
    } else {
        skills
            .iter()
            .map(|skill| skill.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn voice_language(language: &str) -> String {
    match language {
        "ChiCVName" => "Chinese".to_owned(),
        "EngCVName" => "English".to_owned(),
        "JapCVName" => "Japanese".to_owned(),
        "KorCVName" => "Korean".to_owned(),
        language => friendly_label(language.trim_end_matches("CVName")),
    }
}

fn friendly_label(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_uppercase(),
        characters.as_str().to_lowercase()
    )
}

fn summarize(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        normalized
    } else {
        format!(
            "{}...",
            normalized.chars().take(max_chars - 3).collect::<String>()
        )
    }
}

fn format_game_text(text: &str, blackboard: &[BlackboardEntry]) -> String {
    let mut resolved = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start) = remaining.find('{') {
        resolved.push_str(&remaining[..start]);
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find('}') else {
            resolved.push_str(&remaining[start..]);
            remaining = "";
            break;
        };
        let placeholder = &after_start[..end];
        let (expression, pattern) = placeholder
            .split_once(':')
            .map_or((placeholder, None), |(expression, pattern)| {
                (expression, Some(pattern))
            });
        match evaluate_placeholder(expression, blackboard) {
            Some(value) => resolved.push_str(&format_placeholder(value, pattern)),
            None => resolved.push_str(&remaining[start..start + end + 2]),
        }
        remaining = &after_start[end + 1..];
    }
    resolved.push_str(remaining);
    strip_game_markup(&resolved)
}

fn evaluate_placeholder<'a>(
    expression: &str,
    blackboard: &'a [BlackboardEntry],
) -> Option<PlaceholderValue<'a>> {
    if let Some(value) = operand_value(expression, blackboard) {
        return Some(value);
    }

    for operator in ['+', '-', '*', '/'] {
        let Some((left, right)) = expression.split_once(operator) else {
            continue;
        };
        let (PlaceholderValue::Number(left), PlaceholderValue::Number(right)) = (
            operand_value(left, blackboard)?,
            operand_value(right, blackboard)?,
        ) else {
            return None;
        };
        let value = match operator {
            '+' => left + right,
            '-' => left - right,
            '*' => left * right,
            '/' if right != 0.0 => left / right,
            _ => return None,
        };
        return Some(PlaceholderValue::Number(value));
    }
    None
}

enum PlaceholderValue<'a> {
    Number(f64),
    Text(&'a str),
}

fn operand_value<'a>(
    operand: &str,
    blackboard: &'a [BlackboardEntry],
) -> Option<PlaceholderValue<'a>> {
    let operand = operand.trim();
    if let Ok(value) = operand.parse::<f64>() {
        return Some(PlaceholderValue::Number(value));
    }
    let value = blackboard
        .iter()
        .find(|entry| entry.key.eq_ignore_ascii_case(operand))?
        .value
        .as_ref()?;
    Some(match value {
        BlackboardValue::Number(value) => PlaceholderValue::Number(*value),
        BlackboardValue::Text(value) => PlaceholderValue::Text(value),
    })
}

fn format_placeholder(value: PlaceholderValue<'_>, pattern: Option<&str>) -> String {
    let PlaceholderValue::Number(mut value) = value else {
        let PlaceholderValue::Text(value) = value else {
            unreachable!()
        };
        return value.to_owned();
    };
    let pattern = pattern.unwrap_or("");
    let percent = pattern.contains('%');
    if percent {
        value *= 100.0;
    }
    let decimals = pattern
        .split_once('.')
        .map_or(0, |(_, decimals)| decimals.trim_end_matches('%').len());
    let mut formatted = format!("{value:.decimals$}");
    if pattern.contains('#') {
        formatted = formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned();
    }
    if percent {
        formatted.push('%');
    }
    formatted
}

fn strip_game_markup(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;
    while let Some(start) = remaining.find('<') {
        output.push_str(&remaining[..start]);
        let Some(end) = remaining[start..].find('>') else {
            output.push_str(&remaining[start..]);
            return output;
        };
        let tag = &remaining[start..start + end + 1];
        if !tag.starts_with("<@")
            && !tag.starts_with("<#")
            && tag != "</>"
            && !tag.starts_with("<image=")
        {
            output.push_str(tag);
        }
        remaining = &remaining[start + end + 1..];
    }
    output.push_str(remaining);
    output
}

async fn invalid_selection(ctx: Context<'_>) -> Result<(), Error> {
    response::error(ctx, "Select an item from the search suggestions.").await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::OperatorIndexEntry;

    #[test]
    fn filters_choices_and_uses_ids_as_values() {
        let entries = vec![
            OperatorIndexEntry {
                id: "chr_arcane".into(),
                name: "Arcane".into(),
            },
            OperatorIndexEntry {
                id: "chr_ardelia".into(),
                name: "Ardelia".into(),
            },
        ];

        let choices = choices(&entries, "arc");
        assert_eq!(choices.len(), 1);
    }

    #[test]
    fn finds_entries_by_id_or_unique_display_name() {
        let entries = vec![
            OperatorIndexEntry {
                id: "chr_0030_zhuangfy".into(),
                name: "Zhuang Fangyi".into(),
            },
            OperatorIndexEntry {
                id: "chr_0032_lizhiyan".into(),
                name: "Arcane".into(),
            },
        ];

        assert_eq!(
            find_entry(&entries, "Zhuang Fangyi").map(CatalogEntry::id),
            Some("chr_0030_zhuangfy")
        );
        assert_eq!(
            find_entry(&entries, "zhuang fangyi").map(CatalogEntry::id),
            Some("chr_0030_zhuangfy")
        );
        assert_eq!(
            find_entry(&entries, "chr_0032_lizhiyan").map(CatalogEntry::name),
            Some("Arcane")
        );
    }

    #[test]
    fn rejects_ambiguous_display_names() {
        let entries = vec![
            OperatorIndexEntry {
                id: "chr_0002_endminm".into(),
                name: "Endministrator".into(),
            },
            OperatorIndexEntry {
                id: "chr_0003_endminf".into(),
                name: "Endministrator".into(),
            },
        ];

        assert!(find_entry(&entries, "Endministrator").is_none());
        assert_eq!(
            find_entry(&entries, "chr_0003_endminf").map(CatalogEntry::id),
            Some("chr_0003_endminf")
        );
    }

    #[test]
    fn creates_friendly_slug() {
        assert_eq!(friendly_name_id("Brigand's Calling"), "brigands-calling");
        assert_eq!(
            friendly_name_id("Type 42: Solemn Phalanx"),
            "type-42-solemn-phalanx"
        );
    }

    #[test]
    fn suffixes_duplicate_names_in_id_order() {
        let entries = vec![
            OperatorIndexEntry {
                id: "chr_0003".into(),
                name: "Endministrator".into(),
            },
            OperatorIndexEntry {
                id: "chr_0002".into(),
                name: "Endministrator".into(),
            },
        ];

        assert_eq!(entry_slug(&entries[0], &entries), "endministrator-2");
        assert_eq!(entry_slug(&entries[1], &entries), "endministrator-1");
    }

    #[test]
    fn truncates_long_descriptions() {
        assert_eq!(summarize("one   two three", 20), "one two three");
        assert_eq!(summarize("abcdefgh", 6), "abc...");
    }

    #[test]
    fn resolves_equipment_placeholders() {
        let blackboard = vec![
            BlackboardEntry {
                key: "dmg_up".into(),
                value: Some(BlackboardValue::Number(0.24)),
            },
            BlackboardEntry {
                key: "duration".into(),
                value: Some(BlackboardValue::Number(15.0)),
            },
        ];

        assert_eq!(
            format_game_text(
                "DMG <@ba.vup>+{dmg_up:0%}</> for {duration}s against <#ba.crush>Crush</>.",
                &blackboard,
            ),
            "DMG +24% for 15s against Crush."
        );
    }

    #[test]
    fn evaluates_equipment_complement_placeholders() {
        let blackboard = vec![BlackboardEntry {
            key: "dmg_taken_down".into(),
            value: Some(BlackboardValue::Number(0.9)),
        }];

        assert_eq!(
            format_game_text("{1-dmg_taken_down:0%} DMG Reduction", &blackboard),
            "10% DMG Reduction"
        );
    }
}

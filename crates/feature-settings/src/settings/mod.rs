mod language;
mod prefix;
mod social;

use bot_core::serenity::{self, ComponentInteraction};
use bot_core::{BotState, Context, Error, poise};
use serde_json::{Value, json};

const HOME_ID: &str = "settings:home";
const ACCENT_COLOR: u32 = 0x805ad5;

/// Manage your personal settings.
#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn settings(ctx: poise::ApplicationContext<'_, BotState, Error>) -> Result<(), Error> {
    ctx.serenity_context
        .http
        .create_interaction_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &json!({
                "type": 4,
                "data": {"flags": 1 << 6 | 1 << 15, "components": home_components(ctx.data.panel_url.as_deref())},
            }),
            Vec::new(),
        )
        .await?;
    Ok(())
}

fn home_components(panel_url: Option<&str>) -> Vec<Value> {
    let mut components = vec![
        json!({
            "type": 10,
            "content": "## Settings\nThese apply to you in every server.",
        }),
        json!({
            "type": 9,
            "components": [{
                "type": 10,
                "content": "### Social embeds\nChoose how Yin embeds your links from each platform.",
            }],
            "accessory": {
                "type": 2,
                "style": 2,
                "label": "Open",
                "custom_id": social::PAGE_ID,
            },
        }),
    ];
    if let Some(panel_url) = panel_url {
        components.push(json!({
            "type": 1,
            "components": [{
                "type": 2,
                "style": 5,
                "label": "Open dashboard",
                "url": format!("{panel_url}/settings"),
            }],
        }));
    }
    vec![json!({"type": 17, "accent_color": ACCENT_COLOR, "components": components})]
}

pub async fn handle_interaction(
    data: &BotState,
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
) -> Result<(), Error> {
    let components = if interaction.data.custom_id == HOME_ID {
        home_components(data.panel_url.as_deref())
    } else if let Some(components) = social::handle_interaction(data, interaction).await? {
        components
    } else {
        return Ok(());
    };
    ctx.http
        .create_interaction_response(
            interaction.id,
            &interaction.token,
            &json!({"type": 7, "data": {"components": components}}),
            Vec::new(),
        )
        .await?;
    Ok(())
}

/// Manage settings for this server.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    default_member_permissions = "MANAGE_GUILD",
    required_permissions = "MANAGE_GUILD",
    subcommand_required,
    subcommands("prefix", "language", "ladder", "server_social", "custom_commands"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn guild(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

pub use feature_social::SocialDisplay;
pub use language::language;
pub use prefix::{DEFAULT_PREFIX, prefix};
pub use social::{server_social, social_platform_keys, validate_social_platform};

fn ladder() -> bot_core::Command {
    feature_moderation::ladder_command()
}

fn custom_commands() -> bot_core::Command {
    crate::custom_commands::cmd()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_opens_social_page() {
        let home = &home_components(Some("https://panel.example"))[0];
        let section = &home["components"][1];
        assert_eq!(section["accessory"]["custom_id"], social::PAGE_ID);
        assert_eq!(
            home["components"][2]["components"][0]["url"],
            "https://panel.example/settings"
        );
    }

    #[test]
    fn settings_use_view_set_reset_with_required_set_values() {
        for (command, values) in [
            (prefix::prefix(), vec!["prefix"]),
            (language::language(), vec!["language"]),
            (social::server_social(), vec!["platform", "enabled"]),
        ] {
            assert_eq!(
                command
                    .subcommands
                    .iter()
                    .map(|command| command.name.as_str())
                    .collect::<Vec<_>>(),
                ["view", "set", "reset"]
            );
            let view = &command.subcommands[0];
            let set = &command.subcommands[1];
            let reset = &command.subcommands[2];
            assert!(view.parameters.is_empty());
            assert_eq!(
                set.parameters
                    .iter()
                    .map(|parameter| parameter.name.as_str())
                    .collect::<Vec<_>>(),
                values
            );
            assert!(set.parameters.iter().all(|parameter| parameter.required));
            assert!(reset.parameters.iter().all(|parameter| !parameter.required));
            assert!(
                command.subcommands.iter().all(
                    |command| command.slash_action.is_some() && command.prefix_action.is_some()
                )
            );
        }
    }
}

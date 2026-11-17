mod language;
mod prefix;
mod social;

use bot_core::command::{CommandContext, CommandFuture};
use bot_core::{BotState, Error, raw};
use serde_json::{Value, json};
use twilight_http::routing::Route;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::guild::Permissions;

use crate::custom_commands::{self, CustomCommands};

const HOME_ID: &str = "settings:home";
const ACCENT_COLOR: u32 = 0x805ad5;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "settings",
    desc = "Manage your personal settings.",
    contexts = "guild bot_dm private_channel",
    integration_types = "guild_install user_install"
)]
pub struct SettingsCommand;

pub fn run_settings(ctx: &CommandContext) -> CommandFuture<'_> {
    Box::pin(async move {
        raw::request(
            &ctx.state.http,
            &Route::InteractionCallback {
                interaction_id: ctx.interaction.id.get(),
                interaction_token: &ctx.interaction.token,
                with_response: false,
            },
            Some(&json!({
                "type": 4,
                "data": {"flags": 1 << 6 | 1 << 15, "components": home_components(ctx.state.panel_url.as_deref())},
            })),
        )
        .await?;
        Ok(())
    })
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
        json!({
            "type": 9,
            "components": [{
                "type": 10,
                "content": "### Translation language\nChoose the language `/embed` translates posts into.",
            }],
            "accessory": {
                "type": 2,
                "style": 2,
                "label": "Open",
                "custom_id": language::PAGE_ID,
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
    interaction: &Interaction,
    component: &MessageComponentInteractionData,
) -> Result<(), Error> {
    let components = if component.custom_id == HOME_ID {
        home_components(data.panel_url.as_deref())
    } else if let Some(components) =
        social::handle_interaction(data, interaction, component).await?
    {
        components
    } else if let Some(components) =
        language::handle_interaction(data, interaction, component).await?
    {
        components
    } else {
        return Ok(());
    };
    raw::request(
        &data.http,
        &Route::InteractionCallback {
            interaction_id: interaction.id.get(),
            interaction_token: &interaction.token,
            with_response: false,
        },
        Some(&json!({"type": 7, "data": {"components": components}})),
    )
    .await?;
    Ok(())
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "guild",
    desc = "Manage settings for this server.",
    contexts = "guild",
    integration_types = "guild_install",
    default_permissions = "manage_guild"
)]
pub enum GuildCommand {
    #[command(name = "prefix")]
    Prefix(prefix::PrefixCommand),
    #[command(name = "language")]
    Language(language::LanguageCommand),
    #[command(name = "social")]
    Social(social::ServerSocialCommand),
    #[command(name = "commands")]
    Commands(CustomCommands),
}

fn manage_guild() -> Permissions {
    Permissions::MANAGE_GUILD
}

pub fn run_guild(ctx: &CommandContext) -> CommandFuture<'_> {
    Box::pin(async move {
        // Server admins can override default permissions, so check them again here.
        let allowed = ctx
            .interaction
            .member
            .as_ref()
            .and_then(|member| member.permissions)
            .is_some_and(|permissions| {
                permissions.intersects(Permissions::MANAGE_GUILD | Permissions::ADMINISTRATOR)
            });
        if !allowed {
            return ctx
                .error("You are missing permissions to run that command.")
                .await;
        }
        match ctx.parse::<GuildCommand>()? {
            GuildCommand::Prefix(command) => prefix::run(ctx, command).await,
            GuildCommand::Language(command) => language::run(ctx, command).await,
            GuildCommand::Social(command) => social::run(ctx, command).await,
            GuildCommand::Commands(command) => custom_commands::run(ctx, command).await,
        }
    })
}

pub use feature_social::SocialDisplay;
pub use prefix::{DEFAULT_PREFIX, active_prefix};
pub use social::{social_platform_keys, validate_social_platform};

#[cfg(test)]
mod tests {
    use super::*;
    use twilight_model::application::command::CommandOption;

    fn names(options: &[CommandOption]) -> Vec<&str> {
        options.iter().map(|option| option.name.as_str()).collect()
    }

    #[test]
    fn home_opens_settings_pages() {
        let home = &home_components(Some("https://panel.example"))[0];
        assert_eq!(
            home["components"][1]["accessory"]["custom_id"],
            social::PAGE_ID
        );
        assert_eq!(
            home["components"][2]["accessory"]["custom_id"],
            language::PAGE_ID
        );
        assert_eq!(
            home["components"][3]["components"][0]["url"],
            "https://panel.example/settings"
        );
    }

    #[test]
    fn settings_use_view_set_reset_with_required_set_values() {
        let guild = GuildCommand::create_command();
        for (group, values) in [
            ("prefix", vec!["prefix"]),
            ("language", vec!["language"]),
            ("social", vec!["platform", "enabled"]),
        ] {
            let group = guild
                .options
                .iter()
                .find(|option| option.name == group)
                .unwrap();
            let subcommands = group.options.as_deref().unwrap();
            assert_eq!(names(subcommands), ["view", "set", "reset"]);
            let option_list = |index: usize| subcommands[index].options.as_deref().unwrap_or(&[]);
            assert!(option_list(0).is_empty());
            assert_eq!(names(option_list(1)), values);
            assert!(
                option_list(1)
                    .iter()
                    .all(|option| option.required == Some(true))
            );
            assert!(
                option_list(2)
                    .iter()
                    .all(|option| option.required != Some(true))
            );
        }
    }
}

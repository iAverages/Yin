mod language;
mod prefix;
mod social;

use bot_core::{Context, Error, poise};

/// Manage your personal settings.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommand_required,
    subcommands("social"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn settings(_ctx: Context<'_>) -> Result<(), Error> {
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

pub use language::language;
pub use prefix::{DEFAULT_PREFIX, prefix};
pub use social::{server_social, social, social_platform_keys, validate_social_platform};

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
    fn settings_use_view_set_reset_with_required_set_values() {
        for (command, values) in [
            (prefix::prefix(), vec!["prefix"]),
            (language::language(), vec!["language"]),
            (social::social(), vec!["platform", "enabled"]),
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

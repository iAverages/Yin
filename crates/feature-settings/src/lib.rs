mod custom_commands;
mod settings;

use bot_core::command::Command;

pub fn commands() -> Vec<Command> {
    vec![
        Command::slash::<settings::SettingsCommand>(settings::run_settings),
        Command::slash::<settings::GuildCommand>(settings::run_guild),
    ]
}

pub use custom_commands::handle_message;
pub use settings::{
    DEFAULT_PREFIX, SocialDisplay, active_prefix, handle_interaction, social_platform_keys,
    validate_social_platform,
};

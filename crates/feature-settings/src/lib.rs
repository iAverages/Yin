mod custom_commands;
mod settings;

use bot_core::Command;

pub fn commands() -> Vec<Command> {
    vec![settings::settings(), settings::guild()]
}

pub use custom_commands::handle_message;
pub use settings::{
    DEFAULT_PREFIX, SocialDisplay, handle_interaction, social_platform_keys,
    validate_social_platform,
};

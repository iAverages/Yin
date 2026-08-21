mod custom_commands;
mod settings;

use bot_core::Command;

pub fn commands() -> Vec<Command> {
    vec![custom_commands::cmd(), settings::settings()]
}

pub use custom_commands::handle_message;

mod settings;

use bot_core::Command;

pub fn commands() -> Vec<Command> {
    vec![settings::settings()]
}

mod admin;

use bot_core::Command;

pub fn commands() -> Vec<Command> {
    vec![admin::admin()]
}

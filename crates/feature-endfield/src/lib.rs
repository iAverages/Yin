mod client;
mod command;
mod model;

use bot_core::Command;

pub fn commands() -> Vec<Command> {
    vec![command::endfield()]
}

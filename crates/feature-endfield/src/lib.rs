mod client;
mod command;
mod model;

use bot_core::command::Command;

pub fn commands() -> Vec<Command> {
    vec![Command::slash::<command::EndfieldCommand>(command::run)]
}

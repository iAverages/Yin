pub mod audit;
mod commands;
mod ladder;
mod locks;

pub use ladder::ladder_command;
pub use locks::process_due_unlocks;

use bot_core::Command;

pub fn commands() -> Vec<Command> {
    vec![commands::mod_command()]
}

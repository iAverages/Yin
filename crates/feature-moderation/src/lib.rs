pub mod audit;
mod commands;
mod ladder;
mod locks;

pub use ladder::{LadderRuleConfig, ladder_command, validate_ladder_rule_config};
pub use locks::process_due_unlocks;

use bot_core::Command;

pub fn commands() -> Vec<Command> {
    vec![commands::mod_command()]
}

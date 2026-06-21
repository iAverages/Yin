mod info;
mod ping;

use bot_core::Command;

pub fn commands() -> Vec<Command> {
    vec![ping::ping(), info::info(), info::user_context()]
}

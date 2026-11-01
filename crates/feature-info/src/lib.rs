mod info;
mod ping;

use bot_core::command::Command;

pub fn commands() -> Vec<Command> {
    vec![
        Command::slash::<ping::PingCommand>(ping::run),
        Command::slash::<info::InfoCommand>(info::run),
        Command::new(info::user_context_definition(), info::run_user_context),
    ]
}

mod bot;
mod guild;
mod user;

use bot_core::command::{CommandContext, CommandFuture};
use twilight_interactions::command::{CommandModel, CreateCommand};

pub use user::{run_user_context, user_context_definition};

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "info",
    desc = "View information about servers, users, and the bot",
    contexts = "guild bot_dm private_channel",
    integration_types = "guild_install user_install"
)]
pub enum InfoCommand {
    #[command(name = "guild")]
    Guild(guild::GuildCommand),
    #[command(name = "user")]
    User(Box<user::UserCommand>),
    #[command(name = "bot")]
    Bot(bot::BotCommand),
}

pub fn run(ctx: &CommandContext) -> CommandFuture<'_> {
    Box::pin(async move {
        match ctx.parse::<InfoCommand>()? {
            InfoCommand::Guild(_) => guild::run(ctx).await,
            InfoCommand::User(command) => user::run(ctx, *command).await,
            InfoCommand::Bot(_) => bot::run(ctx).await,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_subcommand_keeps_its_options() {
        let definition = InfoCommand::create_command();
        let user = definition
            .options
            .iter()
            .find(|option| option.name == "user")
            .unwrap();
        let names: Vec<_> = user
            .options
            .iter()
            .flatten()
            .map(|option| option.name.as_str())
            .collect();
        assert_eq!(names, ["user", "id"]);
    }
}

use bot_core::command::CommandContext;
use bot_core::response::{Embed, EmbedKind};
use bot_core::{BotState, Error};
use database::CustomCommandRepository;
use database::settings::{CustomCommandName, CustomCommandText};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::Message;
use twilight_model::channel::message::AllowedMentions;

#[derive(CommandModel, CreateCommand)]
#[command(name = "commands", desc = "Manage this server's custom commands.")]
pub enum CustomCommands {
    #[command(name = "create")]
    Create(Create),
    #[command(name = "list")]
    List(List),
    #[command(name = "remove")]
    Remove(Remove),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "create", desc = "Create or replace a custom command.")]
pub struct Create {
    /// Command name
    name: String,
    /// Text the bot will send
    text: String,
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "list", desc = "List this server's custom commands.")]
pub struct List;

#[derive(CommandModel, CreateCommand)]
#[command(name = "remove", desc = "Remove a custom command.")]
pub struct Remove {
    /// Command name
    name: String,
}

pub async fn run(ctx: &CommandContext, command: CustomCommands) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let repository = CustomCommandRepository::new(&ctx.state.database);

    match command {
        CustomCommands::Create(Create { name, text }) => {
            let Some(name) = CustomCommandName::parse(&name) else {
                return ctx
                    .error(
                        "Names must be 1-32 characters using only letters, numbers, `_`, or `-`.",
                    )
                    .await;
            };
            let Some(text) = CustomCommandText::parse(&text) else {
                return ctx.error("Text must be 1-2,000 characters.").await;
            };
            repository.upsert(guild_id.get(), &name, &text).await?;
            ctx.send(
                Embed::new(EmbedKind::Success, "Custom Command Saved").field(
                    "Command",
                    format!("`{name}`"),
                    true,
                ),
            )
            .await
        }
        CustomCommands::List(_) => {
            let commands = repository.list_entries(guild_id.get()).await?;
            if commands.is_empty() {
                return ctx
                    .send(Embed::new(
                        EmbedKind::Info,
                        "No custom commands have been created.",
                    ))
                    .await;
            }
            ctx.send(
                Embed::new(EmbedKind::Info, "Custom Commands").description(
                    commands
                        .into_iter()
                        .map(|command| format!("`{}`", command.name))
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
            )
            .await
        }
        CustomCommands::Remove(Remove { name }) => {
            let Some(name) = CustomCommandName::parse(&name) else {
                return ctx.error("Invalid command name.").await;
            };
            if !repository.remove(guild_id.get(), name.as_str()).await? {
                return ctx.error("That custom command does not exist.").await;
            }
            ctx.send(
                Embed::new(EmbedKind::Success, "Custom Command Removed").field(
                    "Command",
                    format!("`{name}`"),
                    true,
                ),
            )
            .await
        }
    }
}

pub async fn handle_message(data: &BotState, message: &Message, prefix: &str) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }
    let Some(guild_id) = message.guild_id else {
        return Ok(());
    };
    let Some(name) = invocation_name(&message.content, prefix) else {
        return Ok(());
    };
    let Some(response) = CustomCommandRepository::new(&data.database)
        .find(guild_id.get(), name.as_str())
        .await?
    else {
        return Ok(());
    };

    data.http
        .create_message(message.channel_id)
        .content(&response)
        .allowed_mentions(Some(&AllowedMentions::default()))
        .await?;
    Ok(())
}

fn invocation_name(content: &str, prefix: &str) -> Option<CustomCommandName> {
    CustomCommandName::parse(content.strip_prefix(prefix)?.split_whitespace().next()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_parses_command_names() {
        assert_eq!(
            CustomCommandName::parse(" Road-Map ")
                .as_ref()
                .map(CustomCommandName::as_str),
            Some("road-map")
        );
        assert_eq!(CustomCommandName::parse("road map"), None);
        assert_eq!(
            invocation_name("$RoadMap extra", "$")
                .as_ref()
                .map(CustomCommandName::as_str),
            Some("roadmap")
        );
        assert_eq!(invocation_name("!roadmap", "$"), None);
    }
}

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::serenity::{self, CreateAllowedMentions, CreateMessage};
use bot_core::{BotState, Context, Error, poise};
use database::settings::{CustomCommandName, CustomCommandText};
use database::{CustomCommandRepository, GuildSettingsRepository};

use crate::DEFAULT_PREFIX;

#[poise::command(
    slash_command,
    guild_only,
    subcommand_required,
    subcommands("create", "list", "remove"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn cmd(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    slash_command,
    guild_only,
    check = "bot_core::permissions::require_manage_guild",
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Command name"] name: String,
    #[description = "Text the bot will send"] text: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let Some(name) = CustomCommandName::parse(&name) else {
        return response::error(
            ctx,
            "Names must be 1-32 characters using only letters, numbers, `_`, or `-`.",
        )
        .await;
    };
    let Some(text) = CustomCommandText::parse(&text) else {
        return response::error(ctx, "Text must be 1-2,000 characters.").await;
    };

    CustomCommandRepository::new(&ctx.data().database)
        .upsert(guild_id.get(), &name, &text)
        .await?;

    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Custom Command Saved").field(
            "Command",
            format!("`{name}`"),
            true,
        ),
    )
    .await
}

#[poise::command(
    slash_command,
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let commands = CustomCommandRepository::new(&ctx.data().database)
        .list_entries(guild_id.get())
        .await?;
    if commands.is_empty() {
        return response::info(ctx, "No custom commands have been created.").await;
    }

    response::send(
        ctx,
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

#[poise::command(
    slash_command,
    guild_only,
    check = "bot_core::permissions::require_manage_guild",
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Command name"] name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().expect("guild-only command has a guild ID");
    let Some(name) = CustomCommandName::parse(&name) else {
        return response::error(ctx, "Invalid command name.").await;
    };
    let removed = CustomCommandRepository::new(&ctx.data().database)
        .remove(guild_id.get(), name.as_str())
        .await?;
    if !removed {
        return response::error(ctx, "That custom command does not exist.").await;
    }

    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Custom Command Removed").field(
            "Command",
            format!("`{name}`"),
            true,
        ),
    )
    .await
}

pub async fn handle_message(
    data: &BotState,
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }
    let Some(guild_id) = message.guild_id else {
        return Ok(());
    };

    let settings = GuildSettingsRepository::new(&data.database)
        .find_by_guild_id(guild_id.get())
        .await?;
    let prefix = settings
        .and_then(|settings| settings.command_prefix)
        .unwrap_or_else(|| DEFAULT_PREFIX.to_owned());
    let Some(name) = invocation_name(&message.content, &prefix) else {
        return Ok(());
    };
    let Some(response) = CustomCommandRepository::new(&data.database)
        .find(guild_id.get(), name.as_str())
        .await?
    else {
        return Ok(());
    };

    message
        .channel_id
        .send_message(
            ctx,
            CreateMessage::new()
                .content(response)
                .allowed_mentions(CreateAllowedMentions::new()),
        )
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

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::serenity::{self, CreateAllowedMentions, CreateMessage};
use bot_core::{BotState, Context, Error, poise};
use database::{CustomCommandRepository, GuildSettingsRepository};

const DEFAULT_PREFIX: &str = "!";
const MAX_NAME_LEN: usize = 32;
const MAX_RESPONSE_LEN: usize = 2000;

#[poise::command(
    slash_command,
    guild_only,
    subcommands("create", "list", "remove"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn cmd(ctx: Context<'_>) -> Result<(), Error> {
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Custom Commands")
            .description("Use `/cmd create` to create or update a custom command."),
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
pub async fn create(
    ctx: Context<'_>,
    #[description = "Command name"] name: String,
    #[description = "Text the bot will send"] text: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };
    let Some(name) = validate_name(&name) else {
        return response::error(
            ctx,
            "Names must be 1-32 characters using only letters, numbers, `_`, or `-`.",
        )
        .await;
    };
    let text = text.trim();
    if text.is_empty() || text.chars().count() > MAX_RESPONSE_LEN {
        return response::error(ctx, "Text must be 1-2,000 characters.").await;
    }

    CustomCommandRepository::new(&ctx.data().database)
        .upsert(guild_id.get(), &name, text)
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
    let Some(guild_id) = ctx.guild_id() else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };
    let names = CustomCommandRepository::new(&ctx.data().database)
        .list(guild_id.get())
        .await?;
    if names.is_empty() {
        return response::info(ctx, "No custom commands have been created.").await;
    }

    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Custom Commands").description(
            names
                .into_iter()
                .map(|name| format!("`{name}`"))
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
    let Some(guild_id) = ctx.guild_id() else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };
    let Some(name) = validate_name(&name) else {
        return response::error(ctx, "Invalid command name.").await;
    };
    let removed = CustomCommandRepository::new(&ctx.data().database)
        .remove(guild_id.get(), &name)
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
        .find(guild_id.get(), &name)
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

fn validate_name(name: &str) -> Option<String> {
    let name = name.trim().to_ascii_lowercase();
    if name.is_empty()
        || name.len() > MAX_NAME_LEN
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return None;
    }
    Some(name)
}

fn invocation_name(content: &str, prefix: &str) -> Option<String> {
    validate_name(content.strip_prefix(prefix)?.split_whitespace().next()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_parses_command_names() {
        assert_eq!(validate_name(" Road-Map ").as_deref(), Some("road-map"));
        assert_eq!(validate_name("road map"), None);
        assert_eq!(
            invocation_name("$RoadMap extra", "$"),
            Some("roadmap".into())
        );
        assert_eq!(invocation_name("!roadmap", "$"), None);
    }
}

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::time::{format_duration, parse_duration};
use bot_core::{Context, Error, poise};
use database::{ModerationRepository, NewPunishmentLadderRule};

pub fn ladder_command() -> bot_core::Command {
    ladder()
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("list", "add", "remove"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
async fn ladder(ctx: Context<'_>) -> Result<(), Error> {
    response::info(ctx, "Use a ladder subcommand.").await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };
    let rules = ModerationRepository::new(&ctx.data().database)
        .ladder_rules(guild_id.get())
        .await?;
    let description = if rules.is_empty() {
        "No punishment ladder rules configured.".to_owned()
    } else {
        rules
            .iter()
            .map(|rule| {
                let duration = rule.duration_seconds.map_or_else(String::new, |seconds| {
                    format!(
                        " for {}",
                        format_duration(std::time::Duration::from_secs(seconds))
                    )
                });
                format!(
                    "`#{}` {} warnings in {} -> {}{}",
                    rule.id,
                    rule.warning_threshold,
                    format_duration(std::time::Duration::from_secs(rule.window_seconds)),
                    rule.action,
                    duration
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Punishment Ladder").description(description),
    )
    .await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
async fn add(
    ctx: Context<'_>,
    #[description = "Active warning count"] threshold: u32,
    #[description = "Counting window, e.g. 30d"] window: String,
    #[description = "timeout, kick, or ban"] action: String,
    #[description = "Required for timeout"] duration: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return response::error(ctx, "This command can only be used in a server.").await;
    };
    if threshold == 0 {
        return response::error(ctx, "Warning threshold must be greater than zero.").await;
    }
    let window = match parse_duration(&window) {
        Ok(value) => value,
        Err(error) => return response::error(ctx, format!("Invalid window: {error}.")).await,
    };
    let action = action.to_ascii_lowercase();
    if !matches!(action.as_str(), "timeout" | "kick" | "ban") {
        return response::error(ctx, "Action must be timeout, kick, or ban.").await;
    }
    let duration_seconds = match (action.as_str(), duration) {
        ("timeout", Some(value)) => match parse_duration(&value) {
            Ok(value) if value.as_secs() <= 28 * 86_400 => Some(value.as_secs()),
            Ok(_) => return response::error(ctx, "Timeout cannot exceed 28 days.").await,
            Err(error) => return response::error(ctx, format!("Invalid duration: {error}.")).await,
        },
        ("timeout", None) => {
            return response::error(ctx, "Timeout rules require a duration.").await;
        }
        (_, Some(_)) => return response::error(ctx, "Only timeout rules accept a duration.").await,
        (_, None) => None,
    };
    let rule = ModerationRepository::new(&ctx.data().database)
        .create_ladder_rule(NewPunishmentLadderRule {
            guild_id: guild_id.get(),
            warning_threshold: threshold,
            window_seconds: window.as_secs(),
            action: &action,
            duration_seconds,
        })
        .await?;
    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Ladder Rule Added")
            .description(format!("Created rule #{}.", rule.id)),
    )
    .await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
async fn remove(ctx: Context<'_>, #[description = "Rule ID"] rule_id: u64) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let repository = ModerationRepository::new(&ctx.data().database);
    let belongs_to_guild = repository
        .ladder_rules(guild_id.get())
        .await?
        .iter()
        .any(|rule| rule.id == rule_id);
    if belongs_to_guild && repository.delete_ladder_rule(rule_id).await? {
        response::send(ctx, Embed::new(EmbedKind::Success, "Ladder Rule Removed")).await
    } else {
        response::error(ctx, "Ladder rule not found.").await
    }
}

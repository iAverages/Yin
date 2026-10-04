use bot_core::response::{self, Embed, EmbedKind};
use bot_core::time::{format_duration, parse_duration};
use bot_core::{Context, Error, poise};
use database::{ModerationRepository, NewPunishmentLadderRule};

pub(crate) const MAX_TIMEOUT_SECONDS: u64 = 28 * 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LadderRuleConfig {
    pub warning_threshold: u32,
    pub window_seconds: u64,
    pub action: String,
    pub duration_seconds: Option<u64>,
}

pub fn validate_ladder_rule_config(
    warning_threshold: u32,
    window_seconds: u64,
    action: &str,
    duration_seconds: Option<u64>,
) -> Result<LadderRuleConfig, String> {
    if warning_threshold == 0 {
        return Err("Warning threshold must be greater than zero.".to_owned());
    }
    if window_seconds == 0 {
        return Err("Counting window must be greater than zero.".to_owned());
    }

    let action = action.trim().to_ascii_lowercase();
    if !matches!(action.as_str(), "timeout" | "kick" | "ban") {
        return Err("Action must be timeout, kick, or ban.".to_owned());
    }

    match (action.as_str(), duration_seconds) {
        ("timeout", Some(duration)) if duration > 0 && duration <= MAX_TIMEOUT_SECONDS => {}
        ("timeout", _) => {
            return Err("Timeout rules need a duration of 1 second to 28 days.".to_owned());
        }
        (_, Some(_)) => return Err("Only timeout rules accept a duration.".to_owned()),
        (_, None) => {}
    }

    Ok(LadderRuleConfig {
        warning_threshold,
        window_seconds,
        action,
        duration_seconds,
    })
}

pub fn ladder_command() -> bot_core::Command {
    ladder()
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommand_required,
    subcommands("list", "add", "remove"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
async fn ladder(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
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
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let window = match parse_duration(&window) {
        Ok(value) => value,
        Err(error) => return response::error(ctx, format!("Invalid window: {error}.")).await,
    };
    let duration_seconds = match duration {
        Some(value) => match parse_duration(&value) {
            Ok(value) => Some(value.as_secs()),
            Err(error) => return response::error(ctx, format!("Invalid duration: {error}.")).await,
        },
        None => None,
    };
    let config =
        match validate_ladder_rule_config(threshold, window.as_secs(), &action, duration_seconds) {
            Ok(config) => config,
            Err(error) => return response::error(ctx, error).await,
        };
    let rule = ModerationRepository::new(&ctx.data().database)
        .create_ladder_rule(NewPunishmentLadderRule {
            guild_id: guild_id.get(),
            warning_threshold: config.warning_threshold,
            window_seconds: config.window_seconds,
            action: &config.action,
            duration_seconds: config.duration_seconds,
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
    if ModerationRepository::new(&ctx.data().database)
        .delete_ladder_rule(guild_id.get(), rule_id)
        .await?
    {
        response::send(ctx, Embed::new(EmbedKind::Success, "Ladder Rule Removed")).await
    } else {
        response::error(ctx, "Ladder rule not found.").await
    }
}

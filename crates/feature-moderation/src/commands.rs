use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bot_core::response::{self, Embed, EmbedKind};
use bot_core::serenity::{self, ChannelType, EditMember, GuildChannel, Mentionable, Permissions};
use bot_core::time::{format_duration, parse_duration};
use bot_core::{Context, Error, poise};
use database::{ModerationCase, ModerationRepository, NewModerationCase, NewWarn};

use crate::locks::{create_lock, is_public_channel, unlock_operation};

const MAX_TIMEOUT_SECONDS: u64 = 28 * 86_400;

#[poise::command(
    prefix_command,
    slash_command,
    rename = "mod",
    guild_only,
    subcommands(
        "warn",
        "revoke",
        "timeout",
        "timeout_remove",
        "kick",
        "ban",
        "unban",
        "history",
        "case",
        "lock",
        "unlock",
        "lockdown",
        "lockdown_end"
    ),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn mod_command(ctx: Context<'_>) -> Result<(), Error> {
    response::info(ctx, "Use one of the available moderation subcommands.").await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
async fn warn(
    ctx: Context<'_>,
    #[description = "Member to warn"] member: serenity::Member,
    #[rest]
    #[description = "Warning reason"]
    reason: String,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return guild_only_error(ctx).await;
    };
    if !validate_target(ctx, &member).await? {
        return hierarchy_error(ctx).await;
    }
    if reason.trim().is_empty() {
        return response::error(ctx, "A warning reason is required.").await;
    }
    let repository = ModerationRepository::new(&ctx.data().database);
    let result = repository
        .create_warn_and_apply_ladder(NewWarn {
            guild_id: guild_id.get(),
            target_user_id: member.user.id.get(),
            moderator_user_id: ctx.author().id.get(),
            reason: reason.trim(),
        })
        .await?;

    let ladder = if let Some(execution) = result.execution {
        let case_id = execution.case_id.ok_or("ladder execution has no case")?;
        let case = repository
            .case_by_id(case_id)
            .await?
            .ok_or("ladder case not found")?;
        if let Some(required) = ladder_permission(&execution.action)
            && !bot_core::permissions::author_has_guild_permission(ctx, required).await?
        {
            let failure = format!(
                "command actor lacks {} required by ladder action {}",
                permission_name(required),
                execution.action
            );
            repository.mark_case_failed(case_id, &failure).await?;
            repository
                .mark_ladder_execution_failed(execution.id, &failure)
                .await?;
            return response::send(
                ctx,
                Embed::new(EmbedKind::Success, "Warning Created").description(format!(
                    "Warning #{} recorded in case #{}. Ladder action `{}` failed: {}.",
                    result.warning.id, result.case.id, execution.action, failure
                )),
            )
            .await;
        }
        match execute_user_action(ctx.serenity_context(), guild_id, member.user.id, &case).await {
            Ok(()) => {
                repository
                    .mark_case_succeeded(case_id, Some("ladder action applied"))
                    .await?;
                repository
                    .mark_ladder_execution_succeeded(execution.id)
                    .await?;
                format!(
                    " Ladder action `{}` applied as case #{}.",
                    execution.action, case_id
                )
            }
            Err(error) => {
                let failure = error.to_string();
                repository.mark_case_failed(case_id, &failure).await?;
                repository
                    .mark_ladder_execution_failed(execution.id, &failure)
                    .await?;
                return Err(error);
            }
        }
    } else {
        String::new()
    };
    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Warning Created").description(format!(
            "Warning #{} recorded in case #{}.{}",
            result.warning.id, result.case.id, ladder
        )),
    )
    .await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
async fn revoke(
    ctx: Context<'_>,
    #[description = "Member whose warning is being revoked"] member: serenity::Member,
    #[description = "Warning ID"] warning_id: u64,
    #[rest]
    #[description = "Revocation reason"]
    reason: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return guild_only_error(ctx).await;
    };
    if !validate_target(ctx, &member).await? {
        return hierarchy_error(ctx).await;
    }
    let repository = ModerationRepository::new(&ctx.data().database);
    let belongs_to_member = repository
        .warnings_for_target(guild_id.get(), member.user.id.get())
        .await?
        .iter()
        .any(|warning| warning.id == warning_id);
    if !belongs_to_member {
        return response::error(ctx, "Active warning not found for that member.").await;
    }
    let case =
        create_pending_user_case(ctx, member.user.id, "warn_revoke", reason.as_deref(), None)
            .await?;
    let revoked = match repository
        .revoke_warning(warning_id, ctx.author().id.get(), reason.as_deref())
        .await
    {
        Ok(revoked) => revoked,
        Err(error) => {
            repository
                .mark_case_failed(case.id, &error.to_string())
                .await?;
            return Err(error.into());
        }
    };
    if revoked.is_none() {
        repository
            .mark_case_failed(case.id, "warning has already been revoked")
            .await?;
        return response::error(ctx, "That warning has already been revoked.").await;
    }
    repository
        .mark_case_succeeded(case.id, Some("warning revoked"))
        .await?;
    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Warning Revoked")
            .description(format!("Case #{} succeeded.", case.id)),
    )
    .await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS"
)]
async fn timeout(
    ctx: Context<'_>,
    #[description = "Member to time out"] member: serenity::Member,
    #[description = "Duration, e.g. 1d2h"] duration: String,
    #[rest]
    #[description = "Reason"]
    reason: Option<String>,
) -> Result<(), Error> {
    let duration = match timeout_duration(ctx, &duration).await? {
        Some(value) => value,
        None => return Ok(()),
    };
    moderate_member(ctx, member, "timeout", reason, Some(duration)).await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    rename = "remove",
    required_permissions = "MODERATE_MEMBERS",
    required_bot_permissions = "MODERATE_MEMBERS"
)]
async fn timeout_remove(
    ctx: Context<'_>,
    #[description = "Member whose timeout is removed"] member: serenity::Member,
    #[rest]
    #[description = "Reason"]
    reason: Option<String>,
) -> Result<(), Error> {
    moderate_member(ctx, member, "timeout_remove", reason, None).await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "KICK_MEMBERS",
    required_bot_permissions = "KICK_MEMBERS"
)]
async fn kick(
    ctx: Context<'_>,
    #[description = "Member to kick"] member: serenity::Member,
    #[rest]
    #[description = "Reason"]
    reason: Option<String>,
) -> Result<(), Error> {
    moderate_member(ctx, member, "kick", reason, None).await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS"
)]
async fn ban(
    ctx: Context<'_>,
    #[description = "Member to ban"] member: serenity::Member,
    #[rest]
    #[description = "Reason"]
    reason: Option<String>,
) -> Result<(), Error> {
    moderate_member(ctx, member, "ban", reason, None).await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    required_bot_permissions = "BAN_MEMBERS"
)]
async fn unban(
    ctx: Context<'_>,
    #[description = "Banned user ID"] user_id: serenity::UserId,
    #[rest]
    #[description = "Reason"]
    reason: Option<String>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return guild_only_error(ctx).await;
    };
    let case = create_pending_user_case(ctx, user_id, "unban", reason.as_deref(), None).await?;
    finish_user_action(ctx, guild_id, user_id, &case).await
}

async fn moderate_member(
    ctx: Context<'_>,
    member: serenity::Member,
    action: &str,
    reason: Option<String>,
    duration: Option<Duration>,
) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return guild_only_error(ctx).await;
    };
    if !validate_target(ctx, &member).await? {
        return hierarchy_error(ctx).await;
    }
    let case =
        create_pending_user_case(ctx, member.user.id, action, reason.as_deref(), duration).await?;
    finish_user_action(ctx, guild_id, member.user.id, &case).await
}

async fn create_pending_user_case(
    ctx: Context<'_>,
    target: serenity::UserId,
    action: &str,
    reason: Option<&str>,
    duration: Option<Duration>,
) -> Result<ModerationCase, Error> {
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let duration_seconds = duration.map(|value| value.as_secs());
    Ok(ModerationRepository::new(&ctx.data().database)
        .create_pending_case(NewModerationCase {
            guild_id: guild_id.get(),
            target_user_id: Some(target.get()),
            target_channel_id: None,
            actor_user_id: Some(ctx.author().id.get()),
            source: "bot",
            action,
            reason,
            duration_seconds,
            expires_at: duration_seconds.map(|seconds| unix_now().saturating_add(seconds as i64)),
            parent_case_id: None,
        })
        .await?)
}

async fn finish_user_action(
    ctx: Context<'_>,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    case: &ModerationCase,
) -> Result<(), Error> {
    let repository = ModerationRepository::new(&ctx.data().database);
    match execute_user_action(ctx.serenity_context(), guild_id, user_id, case).await {
        Ok(()) => {
            repository.mark_case_succeeded(case.id, None).await?;
            response::send(
                ctx,
                Embed::new(EmbedKind::Success, "Moderation Action Applied")
                    .description(format!("Case #{}: `{}` succeeded.", case.id, case.action)),
            )
            .await
        }
        Err(error) => {
            repository
                .mark_case_failed(case.id, &error.to_string())
                .await?;
            Err(error)
        }
    }
}

async fn execute_user_action(
    serenity: &serenity::Context,
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    case: &ModerationCase,
) -> Result<(), Error> {
    match case.action.as_str() {
        "timeout" => {
            let expires_at = case.expires_at.ok_or("timeout case has no expiry")?;
            let timestamp = serenity::Timestamp::from_unix_timestamp(expires_at)?;
            guild_id
                .edit_member(
                    serenity,
                    user_id,
                    EditMember::new()
                        .disable_communication_until_datetime(timestamp)
                        .audit_log_reason(case.reason.as_deref().unwrap_or("Moderation timeout")),
                )
                .await?;
        }
        "timeout_remove" => {
            guild_id
                .edit_member(
                    serenity,
                    user_id,
                    EditMember::new()
                        .enable_communication()
                        .audit_log_reason(case.reason.as_deref().unwrap_or("Timeout removed")),
                )
                .await?;
        }
        "kick" => {
            guild_id
                .kick_with_reason(
                    serenity,
                    user_id,
                    case.reason.as_deref().unwrap_or("Moderation kick"),
                )
                .await?;
        }
        "ban" => {
            guild_id
                .ban_with_reason(
                    serenity,
                    user_id,
                    0,
                    case.reason.as_deref().unwrap_or("Moderation ban"),
                )
                .await?;
        }
        "unban" => {
            serenity
                .http
                .remove_ban(guild_id, user_id, case.reason.as_deref())
                .await?
        }
        action => return Err(format!("unsupported moderation action: {action}").into()),
    }
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
async fn history(
    ctx: Context<'_>,
    #[description = "Member whose history to view"] member: serenity::Member,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let cases = ModerationRepository::new(&ctx.data().database)
        .cases_for_target(guild_id.get(), member.user.id.get(), 20, 0)
        .await?;
    let description = if cases.is_empty() {
        "No moderation cases found.".to_owned()
    } else {
        cases
            .iter()
            .map(format_case_line)
            .collect::<Vec<_>>()
            .join("\n")
    };
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Moderation History").description(description),
    )
    .await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
async fn case(ctx: Context<'_>, #[description = "Case ID"] case_id: u64) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let repository = ModerationRepository::new(&ctx.data().database);
    let Some(case) = repository.case_by_id(case_id).await? else {
        return response::error(ctx, "Moderation case not found.").await;
    };
    if case.guild_id != guild_id.get() {
        return response::error(ctx, "Moderation case not found.").await;
    }
    let events = repository.case_history(case_id).await?;
    let event_text = events
        .iter()
        .map(|event| {
            format!(
                "`{}` <t:{}:R>{}",
                event.status,
                event.created_at,
                event
                    .detail
                    .as_deref()
                    .map_or_else(String::new, |detail| format!(" - {detail}"))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, format!("Case #{}", case.id))
            .description(format_case_line(&case))
            .field("Events", event_text, false),
    )
    .await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS"
)]
async fn lock(
    ctx: Context<'_>,
    #[description = "Lock duration, e.g. 30m"] duration: Option<String>,
    #[rest]
    #[description = "Reason"]
    reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let channel = match ctx.channel_id().to_channel(ctx.serenity_context()).await? {
        serenity::Channel::Guild(channel)
            if matches!(
                channel.kind,
                ChannelType::Text | ChannelType::News | ChannelType::Forum
            ) =>
        {
            channel
        }
        _ => return response::error(ctx, "This channel cannot be locked.").await,
    };
    lock_channels(ctx, guild_id, vec![channel], "lock", duration, reason).await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS"
)]
async fn unlock(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let repository = ModerationRepository::new(&ctx.data().database);
    let targets = repository
        .active_channel_lock_targets(guild_id.get(), ctx.channel_id().get())
        .await?;
    if targets.is_empty() {
        return response::error(ctx, "This channel has no active bot lock.").await;
    }
    let mut operation_ids = targets
        .iter()
        .map(|target| target.operation_id)
        .collect::<Vec<_>>();
    operation_ids.sort_unstable();
    operation_ids.dedup();
    let mut restored = 0;
    for operation_id in operation_ids {
        let operation = repository
            .claim_channel_lock(operation_id, "manual-command")
            .await?;
        if let Some(operation) = operation {
            let case = create_pending_channel_case(
                ctx,
                ctx.channel_id(),
                "unlock",
                None,
                None,
                Some(operation.case_id),
            )
            .await?;
            unlock_operation(
                ctx.serenity_context().http.as_ref(),
                &ctx.data().database,
                &operation,
                &case,
            )
            .await?;
            restored += 1;
        }
    }
    if restored == 0 {
        return response::error(ctx, "The active lock is already being restored.").await;
    }
    response::send(ctx, Embed::new(EmbedKind::Success, "Channel Unlocked")).await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS"
)]
async fn lockdown(
    ctx: Context<'_>,
    #[description = "Lockdown duration, e.g. 1h"] duration: Option<String>,
    #[rest]
    #[description = "Reason"]
    reason: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let guild = guild_id.to_partial_guild(ctx.serenity_context()).await?;
    let everyone = guild
        .roles
        .get(&guild_id.everyone_role())
        .map_or_else(Permissions::empty, |role| role.permissions);
    let channels = guild_id
        .channels(ctx.serenity_context())
        .await?
        .into_values()
        .filter(|channel| is_public_channel(channel, everyone))
        .collect::<Vec<_>>();
    if channels.is_empty() {
        return response::error(ctx, "No public channels are eligible for lockdown.").await;
    }
    lock_channels(ctx, guild_id, channels, "lockdown", duration, reason).await
}

#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    rename = "end",
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS"
)]
async fn lockdown_end(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    let repository = ModerationRepository::new(&ctx.data().database);
    let operations = repository.active_channel_locks(guild_id.get()).await?;
    let lockdowns = operations
        .into_iter()
        .filter(|operation| operation.action == "lockdown")
        .collect::<Vec<_>>();
    if lockdowns.is_empty() {
        return response::error(ctx, "No active lockdown found.").await;
    }
    let mut restored = 0;
    for operation in lockdowns {
        let operation = repository
            .claim_channel_lock(operation.id, "manual-command")
            .await?;
        if let Some(operation) = operation {
            let case = create_pending_channel_case(
                ctx,
                ctx.channel_id(),
                "lockdown_end",
                None,
                None,
                Some(operation.case_id),
            )
            .await?;
            unlock_operation(
                ctx.serenity_context().http.as_ref(),
                &ctx.data().database,
                &operation,
                &case,
            )
            .await?;
            restored += 1;
        }
    }
    if restored == 0 {
        return response::error(ctx, "The active lockdown is already being restored.").await;
    }
    response::send(ctx, Embed::new(EmbedKind::Success, "Lockdown Ended")).await
}

async fn lock_channels(
    ctx: Context<'_>,
    guild_id: serenity::GuildId,
    channels: Vec<GuildChannel>,
    action: &str,
    duration: Option<String>,
    reason: Option<String>,
) -> Result<(), Error> {
    let duration_seconds = match duration {
        Some(value) => match parse_duration(&value) {
            Ok(value) => Some(value.as_secs()),
            Err(error) => return response::error(ctx, format!("Invalid duration: {error}.")).await,
        },
        None => None,
    };
    let repository = ModerationRepository::new(&ctx.data().database);
    for channel in &channels {
        if !repository
            .active_channel_lock_targets(guild_id.get(), channel.id.get())
            .await?
            .is_empty()
        {
            return response::error(
                ctx,
                format!("{} already has an active bot lock.", channel.id.mention()),
            )
            .await;
        }
    }
    let case = create_pending_channel_case(
        ctx,
        ctx.channel_id(),
        action,
        reason.as_deref(),
        duration_seconds,
        None,
    )
    .await?;
    let operation = match create_lock(crate::locks::LockRequest {
        http: ctx.serenity_context().http.as_ref(),
        database: &ctx.data().database,
        guild_id,
        actor_user_id: ctx.author().id,
        case: &case,
        channels: &channels,
        action,
        reason: reason.as_deref(),
        duration_seconds,
    })
    .await
    {
        Ok(operation) => operation,
        Err(error) => {
            if let Some(database::DatabaseError::ChannelAlreadyLocked { channel_id }) =
                error.downcast_ref::<database::DatabaseError>()
            {
                return response::error(
                    ctx,
                    format!("Channel <#{channel_id}> already has an active bot lock."),
                )
                .await;
            }
            return Err(error);
        }
    };
    response::send(
        ctx,
        Embed::new(
            EmbedKind::Success,
            if action == "lockdown" {
                "Lockdown Started"
            } else {
                "Channel Locked"
            },
        )
        .description(format!(
            "Case #{} / operation #{} applied to {} channel(s).",
            case.id,
            operation.id,
            channels.len()
        )),
    )
    .await
}

async fn create_pending_channel_case(
    ctx: Context<'_>,
    target: serenity::ChannelId,
    action: &str,
    reason: Option<&str>,
    duration_seconds: Option<u64>,
    parent_case_id: Option<u64>,
) -> Result<ModerationCase, Error> {
    let guild_id = ctx.guild_id().ok_or("guild command missing guild")?;
    Ok(ModerationRepository::new(&ctx.data().database)
        .create_pending_case(NewModerationCase {
            guild_id: guild_id.get(),
            target_user_id: None,
            target_channel_id: Some(target.get()),
            actor_user_id: Some(ctx.author().id.get()),
            source: "bot",
            action,
            reason,
            duration_seconds,
            expires_at: duration_seconds.map(|seconds| unix_now().saturating_add(seconds as i64)),
            parent_case_id,
        })
        .await?)
}

async fn validate_target(ctx: Context<'_>, member: &serenity::Member) -> Result<bool, Error> {
    bot_core::permissions::can_moderate_member(ctx, member).await
}

fn ladder_permission(action: &str) -> Option<Permissions> {
    match action {
        "timeout" => Some(Permissions::MODERATE_MEMBERS),
        "kick" => Some(Permissions::KICK_MEMBERS),
        "ban" => Some(Permissions::BAN_MEMBERS),
        _ => None,
    }
}

fn permission_name(permission: Permissions) -> &'static str {
    if permission == Permissions::MODERATE_MEMBERS {
        "MODERATE_MEMBERS"
    } else if permission == Permissions::KICK_MEMBERS {
        "KICK_MEMBERS"
    } else {
        "BAN_MEMBERS"
    }
}

async fn timeout_duration(ctx: Context<'_>, input: &str) -> Result<Option<Duration>, Error> {
    match parse_duration(input) {
        Ok(duration) if duration.as_secs() <= MAX_TIMEOUT_SECONDS => Ok(Some(duration)),
        Ok(_) => {
            response::error(ctx, "Timeout cannot exceed 28 days.").await?;
            Ok(None)
        }
        Err(error) => {
            response::error(ctx, format!("Invalid duration: {error}.")).await?;
            Ok(None)
        }
    }
}

fn format_case_line(case: &ModerationCase) -> String {
    let duration = case.duration_seconds.map_or_else(String::new, |seconds| {
        format!(" ({})", format_duration(Duration::from_secs(seconds)))
    });
    format!(
        "`#{}` **{}**{} - {} <t:{}:R>",
        case.id, case.action, duration, case.status, case.created_at
    )
}

async fn guild_only_error(ctx: Context<'_>) -> Result<(), Error> {
    response::error(ctx, "This command can only be used in a server.").await
}

async fn hierarchy_error(ctx: Context<'_>) -> Result<(), Error> {
    response::error(
        ctx,
        "You and the bot must both have a higher role than that member.",
    )
    .await
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_actions_select_required_actor_permission() {
        assert_eq!(
            ladder_permission("timeout"),
            Some(Permissions::MODERATE_MEMBERS)
        );
        assert_eq!(ladder_permission("kick"), Some(Permissions::KICK_MEMBERS));
        assert_eq!(ladder_permission("ban"), Some(Permissions::BAN_MEMBERS));
        assert_eq!(ladder_permission("unknown"), None);
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

use bot_core::Error;
use bot_core::serenity::{
    self, ChannelType, GuildChannel, PermissionOverwrite, PermissionOverwriteType, Permissions,
};
use database::{
    ChannelLockOperation, ChannelLockTarget, Database, ModerationCase, ModerationRepository,
    NewChannelLockOperation, NewChannelLockTarget, NewModerationCase,
};

const CLAIM_LIMIT: u32 = 25;

pub(crate) fn lock_bits(kind: ChannelType) -> Permissions {
    let mut permissions = Permissions::SEND_MESSAGES;
    if matches!(
        kind,
        ChannelType::Text | ChannelType::News | ChannelType::Forum
    ) {
        permissions |= Permissions::SEND_MESSAGES_IN_THREADS;
    }
    permissions
}

pub(crate) fn everyone_overwrite(
    channel: &GuildChannel,
    guild_id: serenity::GuildId,
) -> Option<&PermissionOverwrite> {
    channel
        .permission_overwrites
        .iter()
        .find(|overwrite| overwrite.kind == PermissionOverwriteType::Role(guild_id.everyone_role()))
}

pub(crate) fn is_public_channel(channel: &GuildChannel, everyone: Permissions) -> bool {
    is_public(
        channel.kind,
        channel.guild_id,
        &channel.permission_overwrites,
        everyone,
    )
}

fn is_public(
    kind: ChannelType,
    guild_id: serenity::GuildId,
    overwrites: &[PermissionOverwrite],
    everyone: Permissions,
) -> bool {
    if !matches!(
        kind,
        ChannelType::Text | ChannelType::News | ChannelType::Forum
    ) {
        return false;
    }
    let overwrite = overwrites.iter().find(|overwrite| {
        overwrite.kind == PermissionOverwriteType::Role(guild_id.everyone_role())
    });
    let allow = overwrite.map_or_else(Permissions::empty, |value| value.allow);
    let deny = overwrite.map_or_else(Permissions::empty, |value| value.deny);
    let effective = (everyone & !deny) | allow;
    effective.contains(Permissions::ADMINISTRATOR) || effective.contains(Permissions::VIEW_CHANNEL)
}

fn locked_overwrite(channel: &GuildChannel, guild_id: serenity::GuildId) -> PermissionOverwrite {
    let previous = everyone_overwrite(channel, guild_id);
    let bits = lock_bits(channel.kind);
    PermissionOverwrite {
        allow: previous.map_or_else(Permissions::empty, |value| value.allow) & !bits,
        deny: previous.map_or_else(Permissions::empty, |value| value.deny) | bits,
        kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
    }
}

pub(crate) struct LockRequest<'a> {
    pub http: &'a serenity::Http,
    pub database: &'a Database,
    pub guild_id: serenity::GuildId,
    pub actor_user_id: serenity::UserId,
    pub case: &'a ModerationCase,
    pub channels: &'a [GuildChannel],
    pub action: &'a str,
    pub reason: Option<&'a str>,
    pub duration_seconds: Option<u64>,
}

pub(crate) async fn create_lock(request: LockRequest<'_>) -> Result<ChannelLockOperation, Error> {
    let result = apply_lock(&request).await;
    if let Err(error) = &result {
        ModerationRepository::new(request.database)
            .mark_case_failed(request.case.id, &error.to_string())
            .await?;
    }
    result
}

async fn apply_lock(request: &LockRequest<'_>) -> Result<ChannelLockOperation, Error> {
    let due_at = request
        .duration_seconds
        .map(|seconds| unix_now().saturating_add(seconds as i64));
    let targets = request
        .channels
        .iter()
        .map(|channel| {
            let previous = everyone_overwrite(channel, request.guild_id);
            NewChannelLockTarget {
                channel_id: channel.id.get(),
                overwrite_target_id: request.guild_id.get(),
                overwrite_target_kind: "role",
                previous_allow: previous.map(|value| value.allow.bits()),
                previous_deny: previous.map(|value| value.deny.bits()),
            }
        })
        .collect::<Vec<_>>();
    let repository = ModerationRepository::new(request.database);
    let operation = repository
        .create_channel_lock(NewChannelLockOperation {
            case_id: request.case.id,
            guild_id: request.guild_id.get(),
            actor_user_id: request.actor_user_id.get(),
            action: request.action,
            reason: request.reason,
            due_at,
            targets: &targets,
        })
        .await?;
    let persisted_targets = repository.channel_lock_targets(operation.id).await?;
    let mut applied: Vec<(GuildChannel, ChannelLockTarget)> = Vec::new();

    for (channel, target) in request.channels.iter().zip(&persisted_targets) {
        if let Err(error) = channel
            .create_permission(request.http, locked_overwrite(channel, request.guild_id))
            .await
        {
            let message = error.to_string();
            repository
                .mark_channel_lock_target_failed(target.id, &message)
                .await?;
            for (applied_channel, applied_target) in applied {
                let restored = match applied_channel.id.to_channel(request.http).await {
                    Ok(serenity::Channel::Guild(current)) => {
                        restore_target(request.http, request.guild_id, &current, &applied_target)
                            .await
                    }
                    Ok(_) => Err("target is no longer a guild channel".into()),
                    Err(error) => Err(error.into()),
                };
                match restored {
                    Ok(()) => {
                        repository
                            .mark_channel_lock_target_restored(applied_target.id)
                            .await?;
                    }
                    Err(error) => {
                        repository
                            .mark_channel_lock_target_failed(
                                applied_target.id,
                                &format!("rollback failed: {error}"),
                            )
                            .await?;
                    }
                }
            }
            repository
                .mark_channel_lock_failed(operation.id, &message)
                .await?;
            return Err(error.into());
        }
        repository
            .mark_channel_lock_target_active(target.id)
            .await?;
        applied.push((channel.clone(), target.clone()));
    }

    let operation = repository
        .mark_channel_lock_active(operation.id)
        .await?
        .ok_or("lock operation did not become active")?;
    repository
        .mark_case_succeeded(request.case.id, Some("channel lock applied"))
        .await?;
    Ok(operation)
}

pub(crate) async fn unlock_operation(
    http: &serenity::Http,
    database: &Database,
    operation: &ChannelLockOperation,
    case: &ModerationCase,
) -> Result<(), Error> {
    let result = restore_operation(http, database, operation).await;
    let repository = ModerationRepository::new(database);
    match &result {
        Ok(()) => {
            repository
                .mark_case_succeeded(case.id, Some("channel lock restored"))
                .await?;
        }
        Err(error) => {
            repository
                .mark_case_failed(case.id, &error.to_string())
                .await?;
            repository
                .mark_channel_lock_restore_failed(operation.id, &error.to_string())
                .await?;
        }
    }
    result
}

async fn restore_operation(
    http: &serenity::Http,
    database: &Database,
    operation: &ChannelLockOperation,
) -> Result<(), Error> {
    let repository = ModerationRepository::new(database);
    let guild_id = serenity::GuildId::new(operation.guild_id);
    let targets = repository.channel_lock_targets(operation.id).await?;
    let mut failures = Vec::new();

    for target in targets
        .iter()
        .filter(|target| matches!(target.status.as_str(), "active" | "restore_failed"))
    {
        let channel = match serenity::ChannelId::new(target.channel_id)
            .to_channel(http)
            .await
        {
            Ok(serenity::Channel::Guild(channel)) => channel,
            Ok(_) => {
                failures.push((target.id, "target is no longer a guild channel".to_owned()));
                continue;
            }
            Err(error) => {
                failures.push((target.id, error.to_string()));
                continue;
            }
        };
        match restore_target(http, guild_id, &channel, target).await {
            Ok(()) => {
                repository
                    .mark_channel_lock_target_restored(target.id)
                    .await?;
            }
            Err(error) => failures.push((target.id, error.to_string())),
        }
    }

    for (target_id, failure) in &failures {
        repository
            .mark_channel_lock_target_failed(*target_id, failure)
            .await?;
    }
    if failures.is_empty() {
        repository.mark_channel_lock_restored(operation.id).await?;
        Ok(())
    } else {
        let message = failures
            .iter()
            .map(|(id, failure)| format!("target {id}: {failure}"))
            .collect::<Vec<_>>()
            .join("; ");
        Err(message.into())
    }
}

async fn restore_target(
    http: &serenity::Http,
    guild_id: serenity::GuildId,
    channel: &GuildChannel,
    target: &ChannelLockTarget,
) -> Result<(), Error> {
    let current = everyone_overwrite(channel, guild_id);
    let current_allow = current.map_or_else(Permissions::empty, |value| value.allow);
    let current_deny = current.map_or_else(Permissions::empty, |value| value.deny);
    let previous_allow = Permissions::from_bits_truncate(target.previous_allow.unwrap_or(0));
    let previous_deny = Permissions::from_bits_truncate(target.previous_deny.unwrap_or(0));
    let bits = lock_bits(channel.kind);
    let applied_allow = previous_allow & !bits;
    let applied_deny = previous_deny | bits;

    if current_allow & bits != applied_allow & bits || current_deny & bits != applied_deny & bits {
        return Err("permission overwrite changed after lock; restore conflict".into());
    }

    let restored_allow = (current_allow & !bits) | (previous_allow & bits);
    let restored_deny = (current_deny & !bits) | (previous_deny & bits);
    if target.previous_allow.is_none()
        && target.previous_deny.is_none()
        && restored_allow.is_empty()
        && restored_deny.is_empty()
    {
        channel
            .delete_permission(
                http,
                PermissionOverwriteType::Role(guild_id.everyone_role()),
            )
            .await?;
    } else {
        channel
            .create_permission(
                http,
                PermissionOverwrite {
                    allow: restored_allow,
                    deny: restored_deny,
                    kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
                },
            )
            .await?;
    }
    Ok(())
}

pub async fn process_due_unlocks(
    http: &serenity::Http,
    database: &Database,
    worker: &str,
) -> Result<usize, Error> {
    let repository = ModerationRepository::new(database);
    let operations = repository
        .claim_due_channel_locks(worker, CLAIM_LIMIT)
        .await?;
    let count = operations.len();
    for operation in operations {
        let target_channel_id = lock_case_channel(&repository, &operation)
            .await?
            .ok_or("lock operation case has no channel target")?;
        let case = repository
            .create_pending_case(NewModerationCase {
                guild_id: operation.guild_id,
                target_user_id: None,
                target_channel_id: Some(target_channel_id),
                actor_user_id: None,
                source: "worker",
                action: unlock_action(&operation),
                reason: Some("scheduled lock expiry"),
                duration_seconds: None,
                expires_at: None,
                parent_case_id: Some(operation.case_id),
            })
            .await?;
        if let Err(error) = unlock_operation(http, database, &operation, &case).await {
            tracing::error!(operation_id = operation.id, error = %error, "due unlock failed");
        }
    }
    Ok(count)
}

async fn lock_case_channel(
    repository: &ModerationRepository<'_>,
    operation: &ChannelLockOperation,
) -> Result<Option<u64>, database::DatabaseError> {
    Ok(repository
        .case_by_id(operation.case_id)
        .await?
        .and_then(|case| case.target_channel_id))
}

fn unlock_action(operation: &ChannelLockOperation) -> &'static str {
    if operation.action == "lockdown" {
        "lockdown_end"
    } else {
        "unlock"
    }
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
    fn lock_bits_include_thread_send_for_message_parents() {
        for kind in [ChannelType::Text, ChannelType::News, ChannelType::Forum] {
            assert!(lock_bits(kind).contains(Permissions::SEND_MESSAGES));
            assert!(lock_bits(kind).contains(Permissions::SEND_MESSAGES_IN_THREADS));
        }
    }

    #[test]
    fn public_selection_applies_everyone_overwrite() {
        let guild_id = serenity::GuildId::new(1);
        let mut overwrites = vec![PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
        }];
        assert!(is_public(
            ChannelType::Text,
            guild_id,
            &[],
            Permissions::VIEW_CHANNEL
        ));
        assert!(!is_public(
            ChannelType::Text,
            guild_id,
            &overwrites,
            Permissions::VIEW_CHANNEL
        ));
        overwrites[0].allow = Permissions::VIEW_CHANNEL;
        assert!(is_public(
            ChannelType::Text,
            guild_id,
            &overwrites,
            Permissions::empty()
        ));
    }

    #[test]
    fn threads_and_non_message_channels_are_not_lockdown_targets() {
        for kind in [
            ChannelType::PublicThread,
            ChannelType::Voice,
            ChannelType::Category,
        ] {
            assert!(!is_public(
                kind,
                serenity::GuildId::new(1),
                &[],
                Permissions::VIEW_CHANNEL
            ));
        }
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

use bot_core::serenity::{self, audit_log};
use database::{Database, ModerationCase, ModerationRepository, NewExternalAuditCase};

const MATCH_WINDOW_SECONDS: i64 = 120;
const AUDIT_PAGE_SIZE: u8 = 100;
/// Bounds the first reconciliation when a guild has no durable cursor yet.
pub const INITIAL_AUDIT_IMPORT_LIMIT: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    Kick,
    Ban,
    Unban,
    Timeout,
    TimeoutRemove,
}

impl AuditAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Kick => "kick",
            Self::Ban => "ban",
            Self::Unban => "unban",
            Self::Timeout => "timeout",
            Self::TimeoutRemove => "timeout_remove",
        }
    }
}

pub fn map_audit_entry(entry: &audit_log::AuditLogEntry) -> Option<AuditAction> {
    map_audit_entry_at(entry, unix_now())
}

pub fn map_audit_entry_at(entry: &audit_log::AuditLogEntry, now_unix: i64) -> Option<AuditAction> {
    match entry.action {
        audit_log::Action::Member(audit_log::MemberAction::Kick) => Some(AuditAction::Kick),
        audit_log::Action::Member(audit_log::MemberAction::BanAdd) => Some(AuditAction::Ban),
        audit_log::Action::Member(audit_log::MemberAction::BanRemove) => Some(AuditAction::Unban),
        audit_log::Action::Member(audit_log::MemberAction::Update) => entry
            .changes
            .as_deref()
            .and_then(|changes| map_member_update_at(changes, now_unix)),
        _ => None,
    }
}

fn map_member_update_at(changes: &[audit_log::Change], now_unix: i64) -> Option<AuditAction> {
    changes.iter().find_map(|change| match change {
        audit_log::Change::CommunicationDisabledUntil { old, new } if old != new => {
            Some(match new {
                Some(timestamp) if timestamp.unix_timestamp() > now_unix => AuditAction::Timeout,
                _ => AuditAction::TimeoutRemove,
            })
        }
        _ => None,
    })
}

pub async fn reconcile_guild_audit_logs(
    http: &serenity::Http,
    database: &Database,
    guild_id: serenity::GuildId,
    initial_import_limit: usize,
) -> Result<usize, bot_core::Error> {
    let repository = ModerationRepository::new(database);
    let cursor = repository.audit_cursor(guild_id.get()).await?;
    let initial_import_limit = if initial_import_limit == 0 {
        INITIAL_AUDIT_IMPORT_LIMIT
    } else {
        initial_import_limit
    };
    let mut before = None;
    let mut entries = Vec::new();

    loop {
        let logs = guild_id
            .audit_logs(http, None, None, before, Some(AUDIT_PAGE_SIZE))
            .await?;
        let page_len = logs.entries.len();
        if page_len == 0 {
            break;
        }
        before = logs.entries.last().map(|entry| entry.id);
        let remaining = cursor.map_or_else(
            || initial_import_limit.saturating_sub(entries.len()),
            |_| usize::MAX,
        );
        let (take, reached_boundary) = page_selection(
            logs.entries.iter().map(|entry| entry.id.get()),
            cursor,
            remaining,
        );
        entries.extend(logs.entries.into_iter().take(take));
        if reached_boundary || page_len < usize::from(AUDIT_PAGE_SIZE) {
            break;
        }
    }

    entries.reverse();
    let mut processed = 0;
    for entry in &entries {
        if process_audit_entry(database, guild_id, entry)
            .await?
            .is_some()
        {
            processed += 1;
        }
        repository
            .advance_audit_cursor(guild_id.get(), entry.id.get())
            .await?;
    }
    Ok(processed)
}

pub async fn reconcile_all_guilds(
    http: &serenity::Http,
    database: &Database,
    guild_ids: impl IntoIterator<Item = serenity::GuildId>,
) -> Result<usize, bot_core::Error> {
    let mut processed = 0;
    for guild_id in guild_ids {
        match reconcile_guild_audit_logs(http, database, guild_id, INITIAL_AUDIT_IMPORT_LIMIT).await
        {
            Ok(count) => processed += count,
            Err(error) => {
                tracing::error!(guild_id = %guild_id, error = %error, "audit reconciliation failed");
            }
        }
    }
    Ok(processed)
}

fn page_selection(
    ids_newest_first: impl IntoIterator<Item = u64>,
    cursor: Option<u64>,
    remaining: usize,
) -> (usize, bool) {
    let mut take = 0;
    for id in ids_newest_first {
        if cursor.is_some_and(|cursor| id <= cursor) || take == remaining {
            return (take, true);
        }
        take += 1;
    }
    (take, take == remaining)
}

pub async fn process_audit_entry(
    database: &Database,
    guild_id: serenity::GuildId,
    entry: &audit_log::AuditLogEntry,
) -> Result<Option<ModerationCase>, database::DatabaseError> {
    let Some(action) = map_audit_entry(entry) else {
        return Ok(None);
    };
    let Some(target_id) = entry.target_id else {
        return Ok(None);
    };
    let new_case = || NewExternalAuditCase {
        guild_id: guild_id.get(),
        target_user_id: Some(target_id.get()),
        target_channel_id: None,
        actor_user_id: Some(entry.user_id.get()),
        external_audit_log_id: entry.id.get(),
        action: action.as_str(),
        reason: entry.reason.as_deref(),
    };
    let repository = ModerationRepository::new(database);
    if let Some(case) = repository
        .attach_external_audit_id(new_case(), unix_now() - MATCH_WINDOW_SECONDS)
        .await?
    {
        return Ok(Some(case));
    }
    Ok(Some(
        repository
            .insert_external_audit_case(new_case())
            .await?
            .case,
    ))
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
    fn maps_direct_member_actions() {
        assert_eq!(
            map_action(audit_log::MemberAction::Kick),
            Some(AuditAction::Kick)
        );
        assert_eq!(
            map_action(audit_log::MemberAction::BanAdd),
            Some(AuditAction::Ban)
        );
        assert_eq!(
            map_action(audit_log::MemberAction::BanRemove),
            Some(AuditAction::Unban)
        );
    }

    #[test]
    fn maps_only_timeout_member_updates() {
        let now = 1_700_000_000;
        let future = serenity::Timestamp::from_unix_timestamp(1_800_000_000).unwrap();
        let past = serenity::Timestamp::from_unix_timestamp(1_600_000_000).unwrap();
        assert_eq!(
            map_member_update_at(
                &[audit_log::Change::CommunicationDisabledUntil {
                    old: None,
                    new: Some(future),
                }],
                now
            ),
            Some(AuditAction::Timeout)
        );
        assert_eq!(
            map_member_update_at(
                &[audit_log::Change::CommunicationDisabledUntil {
                    old: Some(future),
                    new: None,
                }],
                now
            ),
            Some(AuditAction::TimeoutRemove)
        );
        assert_eq!(
            map_member_update_at(
                &[audit_log::Change::CommunicationDisabledUntil {
                    old: Some(future),
                    new: Some(past),
                }],
                now
            ),
            Some(AuditAction::TimeoutRemove)
        );
        assert_eq!(
            map_member_update_at(
                &[audit_log::Change::Nick {
                    old: None,
                    new: Some("name".to_owned()),
                }],
                now
            ),
            None
        );
    }

    #[test]
    fn pagination_stops_at_cursor_and_selects_newer_entries() {
        assert_eq!(
            page_selection([205, 204, 203, 202, 201], Some(202), 500),
            (3, true)
        );
    }

    #[test]
    fn initial_pagination_honors_import_bound() {
        assert_eq!(page_selection((401..=500).rev(), None, 35), (35, true));
        assert_eq!(page_selection((401..=500).rev(), None, 500), (100, false));
    }

    fn map_action(action: audit_log::MemberAction) -> Option<AuditAction> {
        match action {
            audit_log::MemberAction::Kick => Some(AuditAction::Kick),
            audit_log::MemberAction::BanAdd => Some(AuditAction::Ban),
            audit_log::MemberAction::BanRemove => Some(AuditAction::Unban),
            _ => None,
        }
    }
}

use sqlx::{FromRow, MySql, QueryBuilder};

use crate::{Database, DatabaseError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModerationStatus {
    Pending,
    Succeeded,
    Failed,
}

impl ModerationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ModerationCase {
    pub id: u64,
    pub guild_id: u64,
    pub target_user_id: Option<u64>,
    pub target_channel_id: Option<u64>,
    pub actor_user_id: Option<u64>,
    pub source: String,
    pub external_audit_log_id: Option<u64>,
    pub action: String,
    pub reason: Option<String>,
    pub duration_seconds: Option<u64>,
    pub expires_at: Option<i64>,
    pub parent_case_id: Option<u64>,
    pub status: String,
    pub failure_reason: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ModerationCaseHistory {
    pub id: u64,
    pub case_id: u64,
    pub status: String,
    pub detail: Option<String>,
    pub created_at: i64,
}

pub struct NewModerationCase<'a> {
    pub guild_id: u64,
    pub target_user_id: Option<u64>,
    pub target_channel_id: Option<u64>,
    pub actor_user_id: Option<u64>,
    pub source: &'a str,
    pub action: &'a str,
    pub reason: Option<&'a str>,
    pub duration_seconds: Option<u64>,
    pub expires_at: Option<i64>,
    pub parent_case_id: Option<u64>,
}

pub struct NewExternalAuditCase<'a> {
    pub guild_id: u64,
    pub target_user_id: Option<u64>,
    pub target_channel_id: Option<u64>,
    pub actor_user_id: Option<u64>,
    pub external_audit_log_id: u64,
    pub action: &'a str,
    pub reason: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalAuditCaseInsert {
    pub case: ModerationCase,
    pub inserted: bool,
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct Warning {
    pub id: u64,
    pub guild_id: u64,
    pub target_user_id: u64,
    pub moderator_user_id: u64,
    pub case_id: Option<u64>,
    pub reason: String,
    pub created_at: i64,
    pub revoked_at: Option<i64>,
    pub revoked_by_user_id: Option<u64>,
    pub revocation_reason: Option<String>,
}

pub struct NewWarning<'a> {
    pub guild_id: u64,
    pub target_user_id: u64,
    pub moderator_user_id: u64,
    pub case_id: Option<u64>,
    pub reason: &'a str,
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct PunishmentLadderRule {
    pub id: u64,
    pub guild_id: u64,
    pub warning_threshold: u32,
    pub window_seconds: u64,
    pub action: String,
    pub duration_seconds: Option<u64>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct NewPunishmentLadderRule<'a> {
    pub guild_id: u64,
    pub warning_threshold: u32,
    pub window_seconds: u64,
    pub action: &'a str,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct PunishmentLadderExecution {
    pub id: u64,
    pub guild_id: u64,
    pub target_user_id: u64,
    pub rule_id: Option<u64>,
    pub case_id: Option<u64>,
    pub warning_id: u64,
    pub warning_count: u32,
    pub action: String,
    pub status: String,
    pub failure_reason: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

pub struct NewPunishmentLadderExecution<'a> {
    pub guild_id: u64,
    pub target_user_id: u64,
    pub rule_id: Option<u64>,
    pub case_id: Option<u64>,
    pub warning_id: u64,
    pub warning_count: u32,
    pub action: &'a str,
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ChannelLockOperation {
    pub id: u64,
    pub case_id: u64,
    pub guild_id: u64,
    pub actor_user_id: u64,
    pub action: String,
    pub reason: Option<String>,
    pub status: String,
    pub due_at: Option<i64>,
    pub claimed_at: Option<i64>,
    pub claimed_by: Option<String>,
    pub completed_at: Option<i64>,
    pub failure_reason: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ChannelLockTarget {
    pub id: u64,
    pub operation_id: u64,
    pub channel_id: u64,
    pub overwrite_target_id: u64,
    pub overwrite_target_kind: String,
    pub previous_allow: Option<u64>,
    pub previous_deny: Option<u64>,
    pub status: String,
    pub failure_reason: Option<String>,
    pub completed_at: Option<i64>,
}

pub struct NewChannelLockOperation<'a> {
    pub case_id: u64,
    pub guild_id: u64,
    pub actor_user_id: u64,
    pub action: &'a str,
    pub reason: Option<&'a str>,
    pub due_at: Option<i64>,
    pub targets: &'a [NewChannelLockTarget<'a>],
}

pub struct NewChannelLockTarget<'a> {
    pub channel_id: u64,
    pub overwrite_target_id: u64,
    pub overwrite_target_kind: &'a str,
    pub previous_allow: Option<u64>,
    pub previous_deny: Option<u64>,
}

pub struct NewWarn<'a> {
    pub guild_id: u64,
    pub target_user_id: u64,
    pub moderator_user_id: u64,
    pub reason: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarnResult {
    pub case: ModerationCase,
    pub warning: Warning,
    pub execution: Option<PunishmentLadderExecution>,
}

#[derive(FromRow)]
struct LadderCandidate {
    id: u64,
    action: String,
    duration_seconds: Option<u64>,
    warning_count: u32,
}

pub struct ModerationRepository<'a> {
    database: &'a Database,
}

impl<'a> ModerationRepository<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub async fn create_pending_case(
        &self,
        new_case: NewModerationCase<'_>,
    ) -> Result<ModerationCase, DatabaseError> {
        let mut transaction = self.database.pool().begin().await?;
        let result = sqlx::query(
            "INSERT INTO moderation_cases
             (guild_id, target_user_id, target_channel_id, actor_user_id, source, action, reason,
              duration_seconds, expires_at, parent_case_id, status)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, FROM_UNIXTIME(?), ?, 'pending')",
        )
        .bind(new_case.guild_id)
        .bind(new_case.target_user_id)
        .bind(new_case.target_channel_id)
        .bind(new_case.actor_user_id)
        .bind(new_case.source)
        .bind(new_case.action)
        .bind(new_case.reason)
        .bind(new_case.duration_seconds)
        .bind(new_case.expires_at)
        .bind(new_case.parent_case_id)
        .execute(&mut *transaction)
        .await?;
        let case_id = result.last_insert_id();
        insert_case_history(&mut transaction, case_id, ModerationStatus::Pending, None).await?;
        let case = fetch_case(&mut *transaction, case_id).await?;
        transaction.commit().await?;
        Ok(case)
    }

    pub async fn mark_case_succeeded(
        &self,
        case_id: u64,
        detail: Option<&str>,
    ) -> Result<Option<ModerationCase>, DatabaseError> {
        self.finish_case(case_id, ModerationStatus::Succeeded, detail)
            .await
    }

    pub async fn mark_case_failed(
        &self,
        case_id: u64,
        failure_reason: &str,
    ) -> Result<Option<ModerationCase>, DatabaseError> {
        self.finish_case(case_id, ModerationStatus::Failed, Some(failure_reason))
            .await
    }

    async fn finish_case(
        &self,
        case_id: u64,
        status: ModerationStatus,
        detail: Option<&str>,
    ) -> Result<Option<ModerationCase>, DatabaseError> {
        let mut transaction = self.database.pool().begin().await?;
        let result = sqlx::query(
            "UPDATE moderation_cases
             SET status = ?, failure_reason = ?, completed_at = CURRENT_TIMESTAMP(3)
             WHERE id = ? AND status = 'pending'",
        )
        .bind(status.as_str())
        .bind(if status == ModerationStatus::Failed {
            detail
        } else {
            None
        })
        .bind(case_id)
        .execute(&mut *transaction)
        .await?;
        if result.rows_affected() == 0 {
            transaction.rollback().await?;
            return Ok(None);
        }
        insert_case_history(&mut transaction, case_id, status, detail).await?;
        let case = fetch_case(&mut *transaction, case_id).await?;
        transaction.commit().await?;
        Ok(Some(case))
    }

    pub async fn insert_external_audit_case(
        &self,
        new_case: NewExternalAuditCase<'_>,
    ) -> Result<ExternalAuditCaseInsert, DatabaseError> {
        let mut transaction = self.database.pool().begin().await?;
        let result = sqlx::query(
            "INSERT INTO moderation_cases
             (guild_id, target_user_id, target_channel_id, actor_user_id, source, external_audit_log_id,
               action, reason, status, completed_at)
             VALUES (?, ?, ?, ?, 'audit_log', ?, ?, ?, 'succeeded', CURRENT_TIMESTAMP(3))
             ON DUPLICATE KEY UPDATE id = LAST_INSERT_ID(id)",
        )
        .bind(new_case.guild_id)
        .bind(new_case.target_user_id)
        .bind(new_case.target_channel_id)
        .bind(new_case.actor_user_id)
        .bind(new_case.external_audit_log_id)
        .bind(new_case.action)
        .bind(new_case.reason)
        .execute(&mut *transaction)
        .await?;
        let inserted = result.rows_affected() == 1;
        let case_id = result.last_insert_id();
        let case = if inserted {
            insert_case_history(
                &mut transaction,
                case_id,
                ModerationStatus::Succeeded,
                Some("imported from audit log"),
            )
            .await?;
            fetch_case(&mut *transaction, case_id).await?
        } else {
            fetch_case(&mut *transaction, case_id).await?
        };
        transaction.commit().await?;
        Ok(ExternalAuditCaseInsert { case, inserted })
    }

    pub async fn attach_external_audit_id(
        &self,
        audit: NewExternalAuditCase<'_>,
        created_after: i64,
    ) -> Result<Option<ModerationCase>, DatabaseError> {
        let mut transaction = self.database.pool().begin().await?;
        if let Some(existing) = sqlx::query_as::<_, ModerationCase>(CASE_SELECT_BY_AUDIT_ID)
            .bind(audit.guild_id)
            .bind(audit.external_audit_log_id)
            .fetch_optional(&mut *transaction)
            .await?
        {
            transaction.commit().await?;
            return Ok(Some(existing));
        }

        let matched = sqlx::query_as::<_, (u64, String)>(MATCH_AUDIT_CASE_QUERY)
            .bind(audit.guild_id)
            .bind(audit.target_user_id)
            .bind(audit.target_channel_id)
            .bind(audit.actor_user_id)
            .bind(audit.actor_user_id)
            .bind(audit.action)
            .bind(created_after)
            .fetch_optional(&mut *transaction)
            .await?;
        let Some((case_id, previous_status)) = matched else {
            transaction.commit().await?;
            return Ok(None);
        };
        let result = sqlx::query(
            "UPDATE moderation_cases
             SET external_audit_log_id = ?,
                 completed_at = CASE WHEN status = 'pending' THEN CURRENT_TIMESTAMP(3)
                                     ELSE completed_at END,
                 status = CASE WHEN status = 'pending' THEN 'succeeded' ELSE status END
             WHERE id = ?",
        )
        .bind(audit.external_audit_log_id)
        .bind(case_id)
        .execute(&mut *transaction)
        .await?;
        if result.rows_affected() == 1 && previous_status == "pending" {
            insert_case_history(
                &mut transaction,
                case_id,
                ModerationStatus::Succeeded,
                Some("confirmed by audit log"),
            )
            .await?;
        }
        let case = fetch_case(&mut *transaction, case_id).await?;
        transaction.commit().await?;
        Ok(Some(case))
    }

    pub async fn audit_cursor(&self, guild_id: u64) -> Result<Option<u64>, DatabaseError> {
        Ok(sqlx::query_scalar(
            "SELECT highest_audit_entry_id FROM moderation_audit_cursors WHERE guild_id = ?",
        )
        .bind(guild_id)
        .fetch_optional(self.database.pool())
        .await?)
    }

    pub async fn advance_audit_cursor(
        &self,
        guild_id: u64,
        audit_entry_id: u64,
    ) -> Result<u64, DatabaseError> {
        sqlx::query(
            "INSERT INTO moderation_audit_cursors (guild_id, highest_audit_entry_id)
             VALUES (?, ?)
             ON DUPLICATE KEY UPDATE highest_audit_entry_id = GREATEST(
                 highest_audit_entry_id, VALUES(highest_audit_entry_id)
             )",
        )
        .bind(guild_id)
        .bind(audit_entry_id)
        .execute(self.database.pool())
        .await?;
        Ok(self.audit_cursor(guild_id).await?.unwrap_or(audit_entry_id))
    }

    pub async fn case_by_id(&self, case_id: u64) -> Result<Option<ModerationCase>, DatabaseError> {
        Ok(sqlx::query_as::<_, ModerationCase>(CASE_SELECT_BY_ID)
            .bind(case_id)
            .fetch_optional(self.database.pool())
            .await?)
    }

    pub async fn case_history(
        &self,
        case_id: u64,
    ) -> Result<Vec<ModerationCaseHistory>, DatabaseError> {
        Ok(sqlx::query_as::<_, ModerationCaseHistory>(
            "SELECT id, case_id, status, detail,
                    CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at
             FROM moderation_case_history WHERE case_id = ? ORDER BY created_at, id",
        )
        .bind(case_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn cases_for_target(
        &self,
        guild_id: u64,
        target_user_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModerationCase>, DatabaseError> {
        Ok(sqlx::query_as::<_, ModerationCase>(CASE_SELECT_FOR_TARGET)
            .bind(guild_id)
            .bind(target_user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.database.pool())
            .await?)
    }

    pub async fn create_warning(&self, warning: NewWarning<'_>) -> Result<Warning, DatabaseError> {
        let result = sqlx::query(
            "INSERT INTO moderation_warnings
             (guild_id, target_user_id, moderator_user_id, case_id, reason)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(warning.guild_id)
        .bind(warning.target_user_id)
        .bind(warning.moderator_user_id)
        .bind(warning.case_id)
        .bind(warning.reason)
        .execute(self.database.pool())
        .await?;
        fetch_warning(self.database.pool(), result.last_insert_id()).await
    }

    pub async fn create_warn_and_apply_ladder(
        &self,
        warn: NewWarn<'_>,
    ) -> Result<WarnResult, DatabaseError> {
        let mut transaction = self.database.pool().begin().await?;
        sqlx::query(
            "INSERT INTO moderation_warning_subjects (guild_id, target_user_id)
             VALUES (?, ?) ON DUPLICATE KEY UPDATE target_user_id = VALUES(target_user_id)",
        )
        .bind(warn.guild_id)
        .bind(warn.target_user_id)
        .execute(&mut *transaction)
        .await?;
        sqlx::query_scalar::<_, u64>(
            "SELECT target_user_id FROM moderation_warning_subjects
             WHERE guild_id = ? AND target_user_id = ? FOR UPDATE",
        )
        .bind(warn.guild_id)
        .bind(warn.target_user_id)
        .fetch_one(&mut *transaction)
        .await?;

        let case_result = sqlx::query(
            "INSERT INTO moderation_cases
             (guild_id, target_user_id, actor_user_id, source, action, reason, status, completed_at)
             VALUES (?, ?, ?, 'bot', 'warn', ?, 'succeeded', CURRENT_TIMESTAMP(3))",
        )
        .bind(warn.guild_id)
        .bind(warn.target_user_id)
        .bind(warn.moderator_user_id)
        .bind(warn.reason)
        .execute(&mut *transaction)
        .await?;
        let case_id = case_result.last_insert_id();
        insert_case_history(&mut transaction, case_id, ModerationStatus::Succeeded, None).await?;

        let warning_result = sqlx::query(
            "INSERT INTO moderation_warnings
             (guild_id, target_user_id, moderator_user_id, case_id, reason)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(warn.guild_id)
        .bind(warn.target_user_id)
        .bind(warn.moderator_user_id)
        .bind(case_id)
        .bind(warn.reason)
        .execute(&mut *transaction)
        .await?;
        let warning_id = warning_result.last_insert_id();

        let candidate = sqlx::query_as::<_, LadderCandidate>(
            "SELECT r.id, r.action, r.duration_seconds,
                    CAST(COUNT(w.id) AS UNSIGNED) AS warning_count
             FROM punishment_ladder_rules r
             JOIN moderation_warnings w
               ON w.guild_id = r.guild_id AND w.target_user_id = ? AND w.revoked_at IS NULL
              AND w.created_at >= TIMESTAMPADD(
                  SECOND, -CAST(r.window_seconds AS SIGNED), CURRENT_TIMESTAMP(3)
              )
             WHERE r.guild_id = ?
             GROUP BY r.id, r.warning_threshold, r.action, r.duration_seconds
             HAVING warning_count >= r.warning_threshold
                AND warning_count - 1 < r.warning_threshold
             ORDER BY CASE r.action WHEN 'ban' THEN 3 WHEN 'kick' THEN 2
                          WHEN 'timeout' THEN 1 ELSE 0 END DESC,
                      r.warning_threshold DESC, r.id DESC
             LIMIT 1",
        )
        .bind(warn.target_user_id)
        .bind(warn.guild_id)
        .fetch_optional(&mut *transaction)
        .await?;

        let execution = if let Some(candidate) = candidate {
            let punishment_case = sqlx::query(
                "INSERT INTO moderation_cases
                 (guild_id, target_user_id, actor_user_id, source, action, reason,
                  duration_seconds, expires_at, parent_case_id, status)
                 VALUES (?, ?, ?, 'ladder', ?, ?, ?,
                         CASE WHEN ? IS NULL THEN NULL ELSE TIMESTAMPADD(
                             SECOND, CAST(? AS SIGNED), CURRENT_TIMESTAMP(3)
                         ) END, ?, 'pending')",
            )
            .bind(warn.guild_id)
            .bind(warn.target_user_id)
            .bind(warn.moderator_user_id)
            .bind(&candidate.action)
            .bind(warn.reason)
            .bind(candidate.duration_seconds)
            .bind(candidate.duration_seconds)
            .bind(candidate.duration_seconds)
            .bind(case_id)
            .execute(&mut *transaction)
            .await?;
            let punishment_case_id = punishment_case.last_insert_id();
            insert_case_history(
                &mut transaction,
                punishment_case_id,
                ModerationStatus::Pending,
                Some("created by punishment ladder"),
            )
            .await?;
            let result = sqlx::query(
                "INSERT INTO punishment_ladder_executions
                 (guild_id, target_user_id, rule_id, case_id, warning_id, warning_count,
                  action, status)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 'pending')
                 ON DUPLICATE KEY UPDATE id = LAST_INSERT_ID(id)",
            )
            .bind(warn.guild_id)
            .bind(warn.target_user_id)
            .bind(candidate.id)
            .bind(punishment_case_id)
            .bind(warning_id)
            .bind(candidate.warning_count)
            .bind(&candidate.action)
            .execute(&mut *transaction)
            .await?;
            Some(fetch_ladder_execution(&mut *transaction, result.last_insert_id()).await?)
        } else {
            None
        };
        let case = fetch_case(&mut *transaction, case_id).await?;
        let warning = fetch_warning(&mut *transaction, warning_id).await?;
        transaction.commit().await?;
        Ok(WarnResult {
            case,
            warning,
            execution,
        })
    }

    pub async fn revoke_warning(
        &self,
        warning_id: u64,
        revoked_by_user_id: u64,
        reason: Option<&str>,
    ) -> Result<Option<Warning>, DatabaseError> {
        let result = sqlx::query(
            "UPDATE moderation_warnings
             SET revoked_at = CURRENT_TIMESTAMP(3), revoked_by_user_id = ?, revocation_reason = ?
             WHERE id = ? AND revoked_at IS NULL",
        )
        .bind(revoked_by_user_id)
        .bind(reason)
        .bind(warning_id)
        .execute(self.database.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        Ok(Some(fetch_warning(self.database.pool(), warning_id).await?))
    }

    pub async fn active_warning_count(
        &self,
        guild_id: u64,
        target_user_id: u64,
        window_seconds: u64,
    ) -> Result<u64, DatabaseError> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM moderation_warnings
             WHERE guild_id = ? AND target_user_id = ? AND revoked_at IS NULL
               AND created_at >= TIMESTAMPADD(SECOND, -CAST(? AS SIGNED), CURRENT_TIMESTAMP(3))",
        )
        .bind(guild_id)
        .bind(target_user_id)
        .bind(window_seconds)
        .fetch_one(self.database.pool())
        .await?;
        Ok(count as u64)
    }

    pub async fn warnings_for_target(
        &self,
        guild_id: u64,
        target_user_id: u64,
    ) -> Result<Vec<Warning>, DatabaseError> {
        Ok(sqlx::query_as::<_, Warning>(WARNING_SELECT_FOR_TARGET)
            .bind(guild_id)
            .bind(target_user_id)
            .fetch_all(self.database.pool())
            .await?)
    }

    pub async fn create_ladder_rule(
        &self,
        rule: NewPunishmentLadderRule<'_>,
    ) -> Result<PunishmentLadderRule, DatabaseError> {
        let result = sqlx::query(
            "INSERT INTO punishment_ladder_rules
             (guild_id, warning_threshold, window_seconds, action, duration_seconds)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(rule.guild_id)
        .bind(rule.warning_threshold)
        .bind(rule.window_seconds)
        .bind(rule.action)
        .bind(rule.duration_seconds)
        .execute(self.database.pool())
        .await?;
        fetch_ladder_rule(self.database.pool(), result.last_insert_id()).await
    }

    pub async fn update_ladder_rule(
        &self,
        rule_id: u64,
        warning_threshold: u32,
        window_seconds: u64,
        action: &str,
        duration_seconds: Option<u64>,
    ) -> Result<Option<PunishmentLadderRule>, DatabaseError> {
        let result = sqlx::query(
            "UPDATE punishment_ladder_rules
             SET warning_threshold = ?, window_seconds = ?, action = ?, duration_seconds = ?
             WHERE id = ?",
        )
        .bind(warning_threshold)
        .bind(window_seconds)
        .bind(action)
        .bind(duration_seconds)
        .bind(rule_id)
        .execute(self.database.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        Ok(Some(
            fetch_ladder_rule(self.database.pool(), rule_id).await?,
        ))
    }

    pub async fn delete_ladder_rule(&self, rule_id: u64) -> Result<bool, DatabaseError> {
        let result = sqlx::query("DELETE FROM punishment_ladder_rules WHERE id = ?")
            .bind(rule_id)
            .execute(self.database.pool())
            .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn ladder_rules(
        &self,
        guild_id: u64,
    ) -> Result<Vec<PunishmentLadderRule>, DatabaseError> {
        Ok(sqlx::query_as::<_, PunishmentLadderRule>(
            "SELECT id, guild_id, warning_threshold, window_seconds, action, duration_seconds,
                    CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
                    CAST(UNIX_TIMESTAMP(updated_at) AS SIGNED) AS updated_at
             FROM punishment_ladder_rules WHERE guild_id = ? ORDER BY warning_threshold, id",
        )
        .bind(guild_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn cases_for_channel(
        &self,
        guild_id: u64,
        target_channel_id: u64,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModerationCase>, DatabaseError> {
        Ok(sqlx::query_as::<_, ModerationCase>(CASE_SELECT_FOR_CHANNEL)
            .bind(guild_id)
            .bind(target_channel_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.database.pool())
            .await?)
    }

    pub async fn applicable_ladder_rule(
        &self,
        guild_id: u64,
        target_user_id: u64,
    ) -> Result<Option<PunishmentLadderRule>, DatabaseError> {
        Ok(sqlx::query_as::<_, PunishmentLadderRule>(
            "SELECT r.id, r.guild_id, r.warning_threshold, r.window_seconds, r.action,
                    r.duration_seconds,
                    CAST(UNIX_TIMESTAMP(r.created_at) AS SIGNED) AS created_at,
                    CAST(UNIX_TIMESTAMP(r.updated_at) AS SIGNED) AS updated_at
             FROM punishment_ladder_rules r
             WHERE r.guild_id = ? AND r.warning_threshold <= (
                 SELECT COUNT(*) FROM moderation_warnings w
                 WHERE w.guild_id = r.guild_id AND w.target_user_id = ?
                   AND w.revoked_at IS NULL
                   AND w.created_at >= TIMESTAMPADD(
                       SECOND, -CAST(r.window_seconds AS SIGNED), CURRENT_TIMESTAMP(3)
                   )
             )
             ORDER BY CASE r.action WHEN 'ban' THEN 3 WHEN 'kick' THEN 2
                          WHEN 'timeout' THEN 1 ELSE 0 END DESC,
                      r.warning_threshold DESC, r.id DESC
             LIMIT 1",
        )
        .bind(guild_id)
        .bind(target_user_id)
        .fetch_optional(self.database.pool())
        .await?)
    }

    pub async fn create_ladder_execution(
        &self,
        execution: NewPunishmentLadderExecution<'_>,
    ) -> Result<PunishmentLadderExecution, DatabaseError> {
        let result = sqlx::query(
            "INSERT INTO punishment_ladder_executions
             (guild_id, target_user_id, rule_id, case_id, warning_id, warning_count, action, status)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'pending')
             ON DUPLICATE KEY UPDATE id = LAST_INSERT_ID(id)",
        )
        .bind(execution.guild_id)
        .bind(execution.target_user_id)
        .bind(execution.rule_id)
        .bind(execution.case_id)
        .bind(execution.warning_id)
        .bind(execution.warning_count)
        .bind(execution.action)
        .execute(self.database.pool())
        .await?;
        fetch_ladder_execution(self.database.pool(), result.last_insert_id()).await
    }

    pub async fn mark_ladder_execution_succeeded(
        &self,
        execution_id: u64,
    ) -> Result<Option<PunishmentLadderExecution>, DatabaseError> {
        self.finish_ladder_execution(execution_id, ModerationStatus::Succeeded, None)
            .await
    }

    pub async fn mark_ladder_execution_failed(
        &self,
        execution_id: u64,
        failure_reason: &str,
    ) -> Result<Option<PunishmentLadderExecution>, DatabaseError> {
        self.finish_ladder_execution(execution_id, ModerationStatus::Failed, Some(failure_reason))
            .await
    }

    async fn finish_ladder_execution(
        &self,
        execution_id: u64,
        status: ModerationStatus,
        failure_reason: Option<&str>,
    ) -> Result<Option<PunishmentLadderExecution>, DatabaseError> {
        let result = sqlx::query(
            "UPDATE punishment_ladder_executions
             SET status = ?, failure_reason = ?, completed_at = CURRENT_TIMESTAMP(3)
             WHERE id = ? AND status = 'pending'",
        )
        .bind(status.as_str())
        .bind(failure_reason)
        .bind(execution_id)
        .execute(self.database.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        Ok(Some(
            fetch_ladder_execution(self.database.pool(), execution_id).await?,
        ))
    }

    pub async fn ladder_executions_for_target(
        &self,
        guild_id: u64,
        target_user_id: u64,
    ) -> Result<Vec<PunishmentLadderExecution>, DatabaseError> {
        Ok(sqlx::query_as::<_, PunishmentLadderExecution>(
            "SELECT id, guild_id, target_user_id, rule_id, case_id, warning_id, warning_count, action,
                    status, failure_reason,
                    CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
                    CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at
             FROM punishment_ladder_executions
             WHERE guild_id = ? AND target_user_id = ? ORDER BY created_at DESC, id DESC",
        )
        .bind(guild_id)
        .bind(target_user_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn create_channel_lock(
        &self,
        operation: NewChannelLockOperation<'_>,
    ) -> Result<ChannelLockOperation, DatabaseError> {
        let mut transaction = self.database.pool().begin().await?;
        for channel_id in sorted_channel_ids(operation.targets) {
            sqlx::query(
                "INSERT INTO channel_lock_subjects (guild_id, channel_id) VALUES (?, ?)
                 ON DUPLICATE KEY UPDATE channel_id = VALUES(channel_id)",
            )
            .bind(operation.guild_id)
            .bind(channel_id)
            .execute(&mut *transaction)
            .await?;
            sqlx::query_scalar::<_, u64>(
                "SELECT channel_id FROM channel_lock_subjects
                 WHERE guild_id = ? AND channel_id = ? FOR UPDATE",
            )
            .bind(operation.guild_id)
            .bind(channel_id)
            .fetch_one(&mut *transaction)
            .await?;
            let reserved = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(
                     SELECT 1 FROM channel_lock_targets t
                     JOIN channel_lock_operations o ON o.id = t.operation_id
                     WHERE o.guild_id = ? AND t.channel_id = ?
                       AND o.status IN ('pending', 'active', 'unlocking')
                 )",
            )
            .bind(operation.guild_id)
            .bind(channel_id)
            .fetch_one(&mut *transaction)
            .await?;
            if reserved {
                transaction.rollback().await?;
                return Err(DatabaseError::ChannelAlreadyLocked { channel_id });
            }
        }
        let result = sqlx::query(
            "INSERT INTO channel_lock_operations
             (case_id, guild_id, actor_user_id, action, reason, status, due_at)
             VALUES (?, ?, ?, ?, ?, 'pending', FROM_UNIXTIME(?))",
        )
        .bind(operation.case_id)
        .bind(operation.guild_id)
        .bind(operation.actor_user_id)
        .bind(operation.action)
        .bind(operation.reason)
        .bind(operation.due_at)
        .execute(&mut *transaction)
        .await?;
        let operation_id = result.last_insert_id();
        for target in operation.targets {
            sqlx::query(
                "INSERT INTO channel_lock_targets
                 (operation_id, channel_id, overwrite_target_id, overwrite_target_kind,
                  previous_allow, previous_deny, status)
                 VALUES (?, ?, ?, ?, ?, ?, 'pending')",
            )
            .bind(operation_id)
            .bind(target.channel_id)
            .bind(target.overwrite_target_id)
            .bind(target.overwrite_target_kind)
            .bind(target.previous_allow)
            .bind(target.previous_deny)
            .execute(&mut *transaction)
            .await?;
        }
        let operation = fetch_lock_operation(&mut *transaction, operation_id).await?;
        transaction.commit().await?;
        Ok(operation)
    }

    pub async fn channel_lock_by_id(
        &self,
        operation_id: u64,
    ) -> Result<Option<ChannelLockOperation>, DatabaseError> {
        Ok(
            sqlx::query_as::<_, ChannelLockOperation>(LOCK_OPERATION_SELECT_BY_ID)
                .bind(operation_id)
                .fetch_optional(self.database.pool())
                .await?,
        )
    }

    pub async fn channel_lock_targets(
        &self,
        operation_id: u64,
    ) -> Result<Vec<ChannelLockTarget>, DatabaseError> {
        Ok(sqlx::query_as::<_, ChannelLockTarget>(
            "SELECT id, operation_id, channel_id, overwrite_target_id, overwrite_target_kind,
                    previous_allow, previous_deny, status, failure_reason,
                    CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at
             FROM channel_lock_targets WHERE operation_id = ? ORDER BY id",
        )
        .bind(operation_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn claim_due_channel_locks(
        &self,
        worker: &str,
        limit: u32,
    ) -> Result<Vec<ChannelLockOperation>, DatabaseError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut transaction = self.database.pool().begin().await?;
        let ids = sqlx::query_scalar::<_, u64>(
            "SELECT id FROM channel_lock_operations
             WHERE (status = 'active' AND due_at IS NOT NULL
                    AND due_at <= CURRENT_TIMESTAMP(3))
                OR (status = 'unlocking' AND claimed_at <= TIMESTAMPADD(
                    SECOND, -?, CURRENT_TIMESTAMP(3)
                ))
             ORDER BY COALESCE(due_at, claimed_at), id
             LIMIT ? FOR UPDATE SKIP LOCKED",
        )
        .bind(UNLOCK_LEASE_SECONDS)
        .bind(limit)
        .fetch_all(&mut *transaction)
        .await?;
        if ids.is_empty() {
            transaction.commit().await?;
            return Ok(Vec::new());
        }

        let mut update = QueryBuilder::<MySql>::new(
            "UPDATE channel_lock_operations SET status = 'unlocking', \
             claimed_at = CURRENT_TIMESTAMP(3), claimed_by = ",
        );
        update.push_bind(worker).push(" WHERE id IN (");
        let mut separated = update.separated(", ");
        for id in &ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
        update.build().execute(&mut *transaction).await?;

        let mut select =
            QueryBuilder::<MySql>::new(format!("{} WHERE id IN (", LOCK_OPERATION_SELECT));
        let mut separated = select.separated(", ");
        for id in &ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(") ORDER BY due_at, id");
        let claimed = select
            .build_query_as::<ChannelLockOperation>()
            .fetch_all(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(claimed)
    }

    pub async fn mark_channel_lock_target_active(
        &self,
        target_id: u64,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query(
            "UPDATE channel_lock_targets SET status = 'active'
             WHERE id = ? AND status = 'pending'",
        )
        .bind(target_id)
        .execute(self.database.pool())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn mark_channel_lock_target_failed(
        &self,
        target_id: u64,
        failure_reason: &str,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query(
            "UPDATE channel_lock_targets
             SET status = CASE WHEN status = 'active' THEN 'restore_failed' ELSE 'failed' END,
                 failure_reason = ?, completed_at = CURRENT_TIMESTAMP(3)
             WHERE id = ? AND status IN ('pending', 'active')",
        )
        .bind(failure_reason)
        .bind(target_id)
        .execute(self.database.pool())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn mark_channel_lock_target_restored(
        &self,
        target_id: u64,
    ) -> Result<bool, DatabaseError> {
        let result = sqlx::query(
            "UPDATE channel_lock_targets
             SET status = 'restored', completed_at = CURRENT_TIMESTAMP(3)
             WHERE id = ? AND status IN ('active', 'restore_failed')",
        )
        .bind(target_id)
        .execute(self.database.pool())
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn mark_channel_lock_active(
        &self,
        operation_id: u64,
    ) -> Result<Option<ChannelLockOperation>, DatabaseError> {
        let result = sqlx::query(
            "UPDATE channel_lock_operations SET status = 'active'
             WHERE id = ? AND status = 'pending'
               AND EXISTS (
                   SELECT 1 FROM channel_lock_targets WHERE operation_id = ?
               )
               AND NOT EXISTS (
                   SELECT 1 FROM channel_lock_targets
                   WHERE operation_id = ? AND status <> 'active'
               )",
        )
        .bind(operation_id)
        .bind(operation_id)
        .bind(operation_id)
        .execute(self.database.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.channel_lock_by_id(operation_id).await
    }

    pub async fn claim_channel_lock(
        &self,
        operation_id: u64,
        worker: &str,
    ) -> Result<Option<ChannelLockOperation>, DatabaseError> {
        let result = sqlx::query(
            "UPDATE channel_lock_operations
             SET status = 'unlocking', claimed_at = CURRENT_TIMESTAMP(3), claimed_by = ?
             WHERE id = ? AND status = 'active'",
        )
        .bind(worker)
        .bind(operation_id)
        .execute(self.database.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.channel_lock_by_id(operation_id).await
    }

    pub async fn mark_channel_lock_failed(
        &self,
        operation_id: u64,
        failure_reason: &str,
    ) -> Result<Option<ChannelLockOperation>, DatabaseError> {
        let result = sqlx::query(
            "UPDATE channel_lock_operations
             SET status = 'failed', failure_reason = ?, completed_at = CURRENT_TIMESTAMP(3)
             WHERE id = ? AND status = 'pending'",
        )
        .bind(failure_reason)
        .bind(operation_id)
        .execute(self.database.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.channel_lock_by_id(operation_id).await
    }

    pub async fn mark_channel_lock_restore_failed(
        &self,
        operation_id: u64,
        failure_reason: &str,
    ) -> Result<Option<ChannelLockOperation>, DatabaseError> {
        let result = sqlx::query(
            "UPDATE channel_lock_operations
             SET status = 'active', failure_reason = ?,
                 claimed_at = NULL, claimed_by = NULL,
                 due_at = TIMESTAMPADD(SECOND, ?, CURRENT_TIMESTAMP(3))
             WHERE id = ? AND status = 'unlocking'",
        )
        .bind(failure_reason)
        .bind(UNLOCK_RETRY_SECONDS)
        .bind(operation_id)
        .execute(self.database.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.channel_lock_by_id(operation_id).await
    }

    pub async fn mark_channel_lock_restored(
        &self,
        operation_id: u64,
    ) -> Result<Option<ChannelLockOperation>, DatabaseError> {
        let result = sqlx::query(
            "UPDATE channel_lock_operations
              SET status = 'restored', completed_at = CURRENT_TIMESTAMP(3)
             WHERE id = ? AND status IN ('active', 'unlocking', 'failed')
               AND NOT EXISTS (
                   SELECT 1 FROM channel_lock_targets
                   WHERE operation_id = ? AND status IN ('active', 'restore_failed')
               )",
        )
        .bind(operation_id)
        .bind(operation_id)
        .execute(self.database.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.channel_lock_by_id(operation_id).await
    }

    pub async fn active_channel_lock_targets(
        &self,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<Vec<ChannelLockTarget>, DatabaseError> {
        Ok(sqlx::query_as::<_, ChannelLockTarget>(
            "SELECT t.id, t.operation_id, t.channel_id, t.overwrite_target_id,
                    t.overwrite_target_kind, t.previous_allow, t.previous_deny, t.status,
                    t.failure_reason,
                    CAST(UNIX_TIMESTAMP(t.completed_at) AS SIGNED) AS completed_at
             FROM channel_lock_targets t
             JOIN channel_lock_operations o ON o.id = t.operation_id
             WHERE o.guild_id = ? AND t.channel_id = ?
               AND t.status IN ('active', 'restore_failed')
               AND o.status IN ('active', 'unlocking', 'failed')
             ORDER BY t.id",
        )
        .bind(guild_id)
        .bind(channel_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn active_channel_locks(
        &self,
        guild_id: u64,
    ) -> Result<Vec<ChannelLockOperation>, DatabaseError> {
        Ok(sqlx::query_as::<_, ChannelLockOperation>(&format!(
            "{LOCK_OPERATION_SELECT} WHERE guild_id = ?
               AND (status IN ('active', 'unlocking') OR EXISTS (
                   SELECT 1 FROM channel_lock_targets t
                   WHERE t.operation_id = channel_lock_operations.id
                     AND t.status IN ('active', 'restore_failed')
               ))
             ORDER BY created_at, id"
        ))
        .bind(guild_id)
        .fetch_all(self.database.pool())
        .await?)
    }
}

const CASE_SELECT_BY_ID: &str =
    "SELECT id, guild_id, target_user_id, target_channel_id, actor_user_id, source,
            external_audit_log_id, action, reason, duration_seconds,
            CAST(UNIX_TIMESTAMP(expires_at) AS SIGNED) AS expires_at, parent_case_id,
            status, failure_reason,
            CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
            CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at
     FROM moderation_cases WHERE id = ?";
const CASE_SELECT_BY_AUDIT_ID: &str =
    "SELECT id, guild_id, target_user_id, target_channel_id, actor_user_id, source,
            external_audit_log_id, action, reason, duration_seconds,
            CAST(UNIX_TIMESTAMP(expires_at) AS SIGNED) AS expires_at, parent_case_id,
            status, failure_reason,
            CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
            CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at
     FROM moderation_cases WHERE guild_id = ? AND external_audit_log_id = ? FOR UPDATE";
const MATCH_AUDIT_CASE_QUERY: &str = "SELECT id, status FROM moderation_cases
     WHERE guild_id = ? AND target_user_id <=> ? AND target_channel_id <=> ?
       AND (? IS NULL OR actor_user_id = ?)
       AND action = ? AND source = 'bot' AND external_audit_log_id IS NULL
       AND status IN ('pending', 'succeeded')
       AND created_at >= FROM_UNIXTIME(?)
     ORDER BY created_at DESC, id DESC LIMIT 1 FOR UPDATE";
const UNLOCK_LEASE_SECONDS: u64 = 120;
const UNLOCK_RETRY_SECONDS: u64 = 60;
const CASE_SELECT_FOR_TARGET: &str =
    "SELECT id, guild_id, target_user_id, target_channel_id, actor_user_id, source,
            external_audit_log_id, action, reason, duration_seconds,
            CAST(UNIX_TIMESTAMP(expires_at) AS SIGNED) AS expires_at, parent_case_id,
            status, failure_reason,
            CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
            CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at
     FROM moderation_cases WHERE guild_id = ? AND target_user_id = ?
     ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?";
const CASE_SELECT_FOR_CHANNEL: &str =
    "SELECT id, guild_id, target_user_id, target_channel_id, actor_user_id, source,
            external_audit_log_id, action, reason, duration_seconds,
            CAST(UNIX_TIMESTAMP(expires_at) AS SIGNED) AS expires_at, parent_case_id,
            status, failure_reason,
            CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
            CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at
     FROM moderation_cases WHERE guild_id = ? AND target_channel_id = ?
     ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?";
const WARNING_SELECT_FOR_TARGET: &str =
    "SELECT id, guild_id, target_user_id, moderator_user_id, case_id, reason,
            CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
            CAST(UNIX_TIMESTAMP(revoked_at) AS SIGNED) AS revoked_at,
            revoked_by_user_id, revocation_reason
     FROM moderation_warnings WHERE guild_id = ? AND target_user_id = ?
     ORDER BY created_at DESC, id DESC";
const LOCK_OPERATION_SELECT: &str =
    "SELECT id, case_id, guild_id, actor_user_id, action, reason, status,
            CAST(UNIX_TIMESTAMP(due_at) AS SIGNED) AS due_at,
            CAST(UNIX_TIMESTAMP(claimed_at) AS SIGNED) AS claimed_at, claimed_by,
            CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at, failure_reason,
            CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at
     FROM channel_lock_operations";
const LOCK_OPERATION_SELECT_BY_ID: &str =
    "SELECT id, case_id, guild_id, actor_user_id, action, reason, status,
            CAST(UNIX_TIMESTAMP(due_at) AS SIGNED) AS due_at,
            CAST(UNIX_TIMESTAMP(claimed_at) AS SIGNED) AS claimed_at, claimed_by,
            CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at, failure_reason,
            CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at
     FROM channel_lock_operations WHERE id = ?";

async fn insert_case_history(
    transaction: &mut sqlx::Transaction<'_, MySql>,
    case_id: u64,
    status: ModerationStatus,
    detail: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO moderation_case_history (case_id, status, detail) VALUES (?, ?, ?)")
        .bind(case_id)
        .bind(status.as_str())
        .bind(detail)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn fetch_case<'e, E>(executor: E, case_id: u64) -> Result<ModerationCase, DatabaseError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    Ok(sqlx::query_as::<_, ModerationCase>(CASE_SELECT_BY_ID)
        .bind(case_id)
        .fetch_one(executor)
        .await?)
}

async fn fetch_warning<'e, E>(executor: E, warning_id: u64) -> Result<Warning, DatabaseError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    Ok(sqlx::query_as::<_, Warning>(
        "SELECT id, guild_id, target_user_id, moderator_user_id, case_id, reason,
                CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
                CAST(UNIX_TIMESTAMP(revoked_at) AS SIGNED) AS revoked_at,
                revoked_by_user_id, revocation_reason
         FROM moderation_warnings WHERE id = ?",
    )
    .bind(warning_id)
    .fetch_one(executor)
    .await?)
}

async fn fetch_ladder_rule<'e, E>(
    executor: E,
    rule_id: u64,
) -> Result<PunishmentLadderRule, DatabaseError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    Ok(sqlx::query_as::<_, PunishmentLadderRule>(
        "SELECT id, guild_id, warning_threshold, window_seconds, action, duration_seconds,
                CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
                CAST(UNIX_TIMESTAMP(updated_at) AS SIGNED) AS updated_at
         FROM punishment_ladder_rules WHERE id = ?",
    )
    .bind(rule_id)
    .fetch_one(executor)
    .await?)
}

async fn fetch_ladder_execution<'e, E>(
    executor: E,
    execution_id: u64,
) -> Result<PunishmentLadderExecution, DatabaseError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    Ok(sqlx::query_as::<_, PunishmentLadderExecution>(
        "SELECT id, guild_id, target_user_id, rule_id, case_id, warning_id, warning_count, action,
                status, failure_reason,
                CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at,
                CAST(UNIX_TIMESTAMP(completed_at) AS SIGNED) AS completed_at
         FROM punishment_ladder_executions WHERE id = ?",
    )
    .bind(execution_id)
    .fetch_one(executor)
    .await?)
}

async fn fetch_lock_operation<'e, E>(
    executor: E,
    operation_id: u64,
) -> Result<ChannelLockOperation, DatabaseError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    Ok(
        sqlx::query_as::<_, ChannelLockOperation>(LOCK_OPERATION_SELECT_BY_ID)
            .bind(operation_id)
            .fetch_one(executor)
            .await?,
    )
}

fn sorted_channel_ids(targets: &[NewChannelLockTarget<'_>]) -> Vec<u64> {
    let mut channel_ids = targets
        .iter()
        .map(|target| target.channel_id)
        .collect::<Vec<_>>();
    channel_ids.sort_unstable();
    channel_ids.dedup();
    channel_ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moderation_statuses_match_persisted_values() {
        assert_eq!(ModerationStatus::Pending.as_str(), "pending");
        assert_eq!(ModerationStatus::Succeeded.as_str(), "succeeded");
        assert_eq!(ModerationStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn lock_subjects_are_reserved_in_sorted_unique_order() {
        let targets = [
            NewChannelLockTarget {
                channel_id: 9,
                overwrite_target_id: 1,
                overwrite_target_kind: "role",
                previous_allow: None,
                previous_deny: None,
            },
            NewChannelLockTarget {
                channel_id: 3,
                overwrite_target_id: 1,
                overwrite_target_kind: "role",
                previous_allow: None,
                previous_deny: None,
            },
            NewChannelLockTarget {
                channel_id: 9,
                overwrite_target_id: 2,
                overwrite_target_kind: "role",
                previous_allow: None,
                previous_deny: None,
            },
        ];

        assert_eq!(sorted_channel_ids(&targets), vec![3, 9]);
    }

    #[test]
    fn audit_match_requires_actor_only_when_known() {
        assert!(MATCH_AUDIT_CASE_QUERY.contains("(? IS NULL OR actor_user_id = ?)"));
        assert_eq!(UNLOCK_LEASE_SECONDS, 120);
        assert_eq!(UNLOCK_RETRY_SECONDS, 60);
    }
}

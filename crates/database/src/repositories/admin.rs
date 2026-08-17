use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};

use crate::{Database, DatabaseError};

const ADMIN_DISCORD_ID: &str = "307952129958477824";
pub const RUNTIME_STALE_SECONDS: i64 = 45;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardSnapshot {
    pub id: u32,
    pub stage: String,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub bot_user_id: Option<String>,
    pub bot_name: Option<String>,
    pub environment: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub shard_count: Option<u32>,
    pub shards: Vec<ShardSnapshot>,
    pub cached_guilds: usize,
    pub unavailable_guilds: usize,
    pub cached_users: usize,
    pub cached_channels: usize,
}

#[derive(Debug, FromRow)]
pub struct RuntimeRecord {
    pub instance_id: String,
    pub snapshot: Json<RuntimeSnapshot>,
    pub stopped: bool,
    pub age_seconds: i64,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminGuild {
    pub id: String,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub shard_id: u32,
}

pub struct AdminRepository<'a> {
    database: &'a Database,
}

impl<'a> AdminRepository<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    /// better auth user ids are not discord ids. only a linked discord oauth
    /// account can grant access
    pub async fn is_admin(&self, user_id: &str) -> Result<bool, DatabaseError> {
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM account WHERE userId = ? AND providerId = 'discord' AND accountId = ?",
        )
        .bind(user_id)
        .bind(ADMIN_DISCORD_ID)
        .fetch_one(self.database.pool())
        .await? > 0)
    }

    pub async fn guilds(&self) -> Result<Vec<AdminGuild>, DatabaseError> {
        Ok(sqlx::query_as(
            "SELECT CAST(guild_id AS CHAR) AS id, name, icon, shard_id FROM bot_guilds
             WHERE bot_present = TRUE ORDER BY COALESCE(name, ''), guild_id",
        )
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn publish(
        &self,
        instance_id: &str,
        snapshot: &RuntimeSnapshot,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            "INSERT INTO bot_runtime (instance_id, snapshot) VALUES (?, ?)
             ON DUPLICATE KEY UPDATE snapshot = VALUES(snapshot), stopped = FALSE, updated_at = CURRENT_TIMESTAMP(3)",
        )
        .bind(instance_id)
        .bind(Json(snapshot))
        .execute(self.database.pool())
        .await?;
        Ok(())
    }

    pub async fn mark_stopped(&self, instance_id: &str) -> Result<(), DatabaseError> {
        sqlx::query("UPDATE bot_runtime SET stopped = TRUE WHERE instance_id = ?")
            .bind(instance_id)
            .execute(self.database.pool())
            .await?;
        Ok(())
    }

    pub async fn runtimes(&self) -> Result<Vec<RuntimeRecord>, DatabaseError> {
        Ok(sqlx::query_as(
            "SELECT instance_id, snapshot, stopped,
                    TIMESTAMPDIFF(SECOND, updated_at, CURRENT_TIMESTAMP(3)) AS age_seconds
             FROM bot_runtime WHERE updated_at >= TIMESTAMPADD(HOUR, -24, CURRENT_TIMESTAMP(3))
             ORDER BY updated_at DESC, instance_id",
        )
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn prune_runtimes(&self) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM bot_runtime WHERE updated_at < TIMESTAMPADD(HOUR, -24, CURRENT_TIMESTAMP(3))")
            .execute(self.database.pool()).await?;
        Ok(())
    }
}

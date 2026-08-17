use sqlx::{FromRow, MySql, QueryBuilder};

use crate::{Database, DatabaseError};

#[derive(Debug, FromRow)]
pub struct BotGuild {
    pub guild_id: u64,
    pub name: Option<String>,
    pub icon: Option<String>,
}

pub struct GuildDirectoryRepository<'a> {
    database: &'a Database,
}

impl<'a> GuildDirectoryRepository<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub async fn upsert_bot_guild(
        &self,
        id: u64,
        name: &str,
        icon: Option<&str>,
        shard_id: u32,
    ) -> Result<(), DatabaseError> {
        sqlx::query("INSERT INTO bot_guilds (guild_id, name, icon, shard_id, bot_present) VALUES (?, ?, ?, ?, TRUE)
            ON DUPLICATE KEY UPDATE name = VALUES(name), icon = VALUES(icon), shard_id = VALUES(shard_id), bot_present = TRUE")
            .bind(id).bind(name).bind(icon).bind(shard_id)
            .execute(self.database.pool()).await?;
        Ok(())
    }

    pub async fn mark_left(&self, id: u64) -> Result<(), DatabaseError> {
        sqlx::query("UPDATE bot_guilds SET bot_present = FALSE WHERE guild_id = ?")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        Ok(())
    }

    pub async fn reconcile_shard(&self, shard_id: u32, ids: &[u64]) -> Result<(), DatabaseError> {
        // batch to avoid bind limit
        const BATCH_SIZE: usize = 30_000;

        let mut tx = self.database.pool().begin().await?;
        sqlx::query("UPDATE bot_guilds SET bot_present = FALSE WHERE shard_id = ?")
            .bind(shard_id)
            .execute(&mut *tx)
            .await?;

        for batch in ids.chunks(BATCH_SIZE) {
            let mut present = QueryBuilder::<MySql>::new(
                "INSERT INTO bot_guilds (guild_id, shard_id, bot_present) ",
            );
            present.push_values(batch, |mut row, id| {
                row.push_bind(id).push_bind(shard_id).push("TRUE");
            });
            present
                .push(" ON DUPLICATE KEY UPDATE shard_id = VALUES(shard_id), bot_present = TRUE");
            present.build().execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn installed(&self) -> Result<Vec<BotGuild>, DatabaseError> {
        Ok(
            sqlx::query_as("SELECT guild_id, name, icon FROM bot_guilds WHERE bot_present = TRUE")
                .fetch_all(self.database.pool())
                .await?,
        )
    }

    pub async fn installed_guild(&self, guild_id: u64) -> Result<Option<BotGuild>, DatabaseError> {
        Ok(sqlx::query_as(
            "SELECT guild_id, name, icon FROM bot_guilds WHERE guild_id = ? AND bot_present = TRUE",
        )
        .bind(guild_id)
        .fetch_optional(self.database.pool())
        .await?)
    }

    pub async fn cached_user_guilds(&self, user_id: &str) -> Result<Option<String>, DatabaseError> {
        Ok(sqlx::query_scalar("SELECT CAST(guilds AS CHAR) FROM user_guild_cache WHERE user_id = ? AND expires_at > CURRENT_TIMESTAMP(3)")
            .bind(user_id).fetch_optional(self.database.pool()).await?)
    }

    pub async fn cache_user_guilds(
        &self,
        user_id: &str,
        guilds: &str,
        ttl_seconds: u32,
    ) -> Result<(), DatabaseError> {
        sqlx::query("INSERT INTO user_guild_cache (user_id, guilds, expires_at) VALUES (?, ?, TIMESTAMPADD(SECOND, ?, CURRENT_TIMESTAMP(3)))
            ON DUPLICATE KEY UPDATE guilds = VALUES(guilds), expires_at = VALUES(expires_at)")
            .bind(user_id).bind(guilds).bind(ttl_seconds).execute(self.database.pool()).await?;
        Ok(())
    }

    pub async fn discord_token(&self, user_id: &str) -> Result<Option<String>, DatabaseError> {
        let token: Option<Option<String>> = sqlx::query_scalar("SELECT accessToken FROM account WHERE userId = ? AND providerId = 'discord' ORDER BY createdAt DESC LIMIT 1")
            .bind(user_id).fetch_optional(self.database.pool()).await?;
        Ok(token.flatten().filter(|token| !token.is_empty()))
    }
}

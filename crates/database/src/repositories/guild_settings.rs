use sea_query::{Alias, Expr, Query, SelectStatement};
use sqlx::Row;

use crate::{Database, DatabaseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildSettings {
    pub guild_id: u64,
    pub command_prefix: Option<String>,
}

pub struct GuildSettingsRepository<'a> {
    database: &'a Database,
}

impl<'a> GuildSettingsRepository<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub async fn find_by_guild_id(
        &self,
        guild_id: u64,
    ) -> Result<Option<GuildSettings>, DatabaseError> {
        let (sql, _) =
            guild_settings_by_guild_id_query(guild_id).build(sea_query::MysqlQueryBuilder);

        let row = sqlx::query(&sql)
            .bind(guild_id)
            .fetch_optional(self.database.pool())
            .await?;

        Ok(row.map(|row| GuildSettings {
            guild_id: row.get("guild_id"),
            command_prefix: row.get("command_prefix"),
        }))
    }

    pub async fn upsert_prefix(&self, guild_id: u64, prefix: &str) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO guild_settings (guild_id, command_prefix)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE command_prefix = VALUES(command_prefix)
            "#,
        )
        .bind(guild_id)
        .bind(prefix)
        .execute(self.database.pool())
        .await?;

        Ok(())
    }

    pub async fn clear_prefix(&self, guild_id: u64) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO guild_settings (guild_id, command_prefix)
            VALUES (?, NULL)
            ON DUPLICATE KEY UPDATE command_prefix = NULL
            "#,
        )
        .bind(guild_id)
        .execute(self.database.pool())
        .await?;

        Ok(())
    }
}

fn guild_settings_by_guild_id_query(guild_id: u64) -> SelectStatement {
    Query::select()
        .columns([Alias::new("guild_id"), Alias::new("command_prefix")])
        .from(Alias::new("guild_settings"))
        .and_where(Expr::col(Alias::new("guild_id")).eq(guild_id))
        .to_owned()
}

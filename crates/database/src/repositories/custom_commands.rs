use sqlx::FromRow;

use crate::settings::{CustomCommandName, CustomCommandText};
use crate::{Database, DatabaseError};

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct CustomCommand {
    pub guild_id: u64,
    pub name: String,
    pub response: String,
}

pub struct CustomCommandRepository<'a> {
    database: &'a Database,
}

impl<'a> CustomCommandRepository<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub async fn find(&self, guild_id: u64, name: &str) -> Result<Option<String>, DatabaseError> {
        Ok(sqlx::query_scalar(
            "SELECT response FROM custom_commands WHERE guild_id = ? AND name = ?",
        )
        .bind(guild_id)
        .bind(name)
        .fetch_optional(self.database.pool())
        .await?)
    }

    pub async fn list_entries(&self, guild_id: u64) -> Result<Vec<CustomCommand>, DatabaseError> {
        Ok(sqlx::query_as::<_, CustomCommand>(
            "SELECT guild_id, name, response FROM custom_commands WHERE guild_id = ? ORDER BY name LIMIT 100",
        )
        .bind(guild_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn upsert(
        &self,
        guild_id: u64,
        name: &CustomCommandName,
        response: &CustomCommandText,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO custom_commands (guild_id, name, response)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE response = VALUES(response)
            "#,
        )
        .bind(guild_id)
        .bind(name.as_str())
        .bind(response.as_str())
        .execute(self.database.pool())
        .await?;

        Ok(())
    }

    pub async fn remove(&self, guild_id: u64, name: &str) -> Result<bool, DatabaseError> {
        let result = sqlx::query("DELETE FROM custom_commands WHERE guild_id = ? AND name = ?")
            .bind(guild_id)
            .bind(name)
            .execute(self.database.pool())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

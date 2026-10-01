use sea_query::{Alias, Expr, Query, SelectStatement};
use sqlx::Row;

use crate::settings::{CommandPrefix, TranslationLanguage};
use crate::{Database, DatabaseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildSettings {
    pub guild_id: u64,
    pub command_prefix: Option<String>,
    pub translation_language: Option<String>,
}

pub struct GuildSettingsRepository<'a> {
    database: &'a Database,
}

impl<'a> GuildSettingsRepository<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub async fn disabled_social_platforms(
        &self,
        guild_id: u64,
    ) -> Result<Vec<String>, DatabaseError> {
        Ok(sqlx::query_scalar(
            "SELECT platform FROM guild_social_embed_opt_outs WHERE guild_id = ? ORDER BY platform",
        )
        .bind(guild_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn set_social_embed_enabled(
        &self,
        guild_id: u64,
        platform: &str,
        enabled: bool,
    ) -> Result<(), DatabaseError> {
        let sql = if enabled {
            "DELETE FROM guild_social_embed_opt_outs WHERE guild_id = ? AND platform = ?"
        } else {
            "INSERT INTO guild_social_embed_opt_outs (guild_id, platform) VALUES (?, ?) \
             ON DUPLICATE KEY UPDATE platform = VALUES(platform)"
        };
        sqlx::query(sql)
            .bind(guild_id)
            .bind(platform)
            .execute(self.database.pool())
            .await?;
        Ok(())
    }

    pub async fn reset_social_embeds(
        &self,
        guild_id: u64,
        platform: Option<&str>,
    ) -> Result<(), DatabaseError> {
        if let Some(platform) = platform {
            return self
                .set_social_embed_enabled(guild_id, platform, true)
                .await;
        }
        sqlx::query("DELETE FROM guild_social_embed_opt_outs WHERE guild_id = ?")
            .bind(guild_id)
            .execute(self.database.pool())
            .await?;
        Ok(())
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
            translation_language: row.get("translation_language"),
        }))
    }

    pub async fn upsert_prefix(
        &self,
        guild_id: u64,
        prefix: &CommandPrefix,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO guild_settings (guild_id, command_prefix)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE command_prefix = VALUES(command_prefix)
            "#,
        )
        .bind(guild_id)
        .bind(prefix.as_str())
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

    pub async fn upsert_translation_language(
        &self,
        guild_id: u64,
        language: &TranslationLanguage,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO guild_settings (guild_id, translation_language)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE translation_language = VALUES(translation_language)
            "#,
        )
        .bind(guild_id)
        .bind(language.as_str())
        .execute(self.database.pool())
        .await?;

        Ok(())
    }

    pub async fn clear_translation_language(&self, guild_id: u64) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT INTO guild_settings (guild_id, translation_language)
            VALUES (?, NULL)
            ON DUPLICATE KEY UPDATE translation_language = NULL
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
        .columns([
            Alias::new("guild_id"),
            Alias::new("command_prefix"),
            Alias::new("translation_language"),
        ])
        .from(Alias::new("guild_settings"))
        .and_where(Expr::col(Alias::new("guild_id")).eq(guild_id))
        .to_owned()
}

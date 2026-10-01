use crate::{Database, DatabaseError};

pub struct UserSocialEmbedsRepository<'a> {
    database: &'a Database,
}

impl<'a> UserSocialEmbedsRepository<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub async fn disabled_platforms(&self, user_id: u64) -> Result<Vec<String>, DatabaseError> {
        Ok(sqlx::query_scalar(
            "SELECT platform FROM user_social_embed_opt_outs WHERE user_id = ? ORDER BY platform",
        )
        .bind(user_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn effective_disabled_platforms(
        &self,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> Result<Vec<String>, DatabaseError> {
        Ok(sqlx::query_scalar(
            "SELECT platform FROM user_social_embed_opt_outs WHERE user_id = ? \
             UNION SELECT platform FROM guild_social_embed_opt_outs WHERE guild_id = ? \
             ORDER BY platform",
        )
        .bind(user_id)
        .bind(guild_id)
        .fetch_all(self.database.pool())
        .await?)
    }

    pub async fn set_enabled(
        &self,
        user_id: u64,
        platform: &str,
        enabled: bool,
    ) -> Result<(), DatabaseError> {
        let sql = if enabled {
            "DELETE FROM user_social_embed_opt_outs WHERE user_id = ? AND platform = ?"
        } else {
            "INSERT INTO user_social_embed_opt_outs (user_id, platform) VALUES (?, ?) \
             ON DUPLICATE KEY UPDATE platform = VALUES(platform)"
        };
        sqlx::query(sql)
            .bind(user_id)
            .bind(platform)
            .execute(self.database.pool())
            .await?;
        Ok(())
    }

    pub async fn reset(&self, user_id: u64, platform: Option<&str>) -> Result<(), DatabaseError> {
        if let Some(platform) = platform {
            return self.set_enabled(user_id, platform, true).await;
        }
        sqlx::query("DELETE FROM user_social_embed_opt_outs WHERE user_id = ?")
            .bind(user_id)
            .execute(self.database.pool())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GuildSettingsRepository;

    #[sqlx::test(migrations = "./migrations")]
    #[ignore = "requires DATABASE_URL with permission to create test databases"]
    async fn server_and_user_opt_outs_apply_independently(
        pool: sqlx::MySqlPool,
    ) -> Result<(), DatabaseError> {
        let database = Database { pool };
        let users = UserSocialEmbedsRepository::new(&database);
        let guilds = GuildSettingsRepository::new(&database);
        assert!(guilds.disabled_social_platforms(10).await?.is_empty());
        assert!(
            users
                .effective_disabled_platforms(1, Some(10))
                .await?
                .is_empty()
        );

        guilds
            .set_social_embed_enabled(10, "twitter", false)
            .await?;
        guilds
            .set_social_embed_enabled(10, "twitter", false)
            .await?;
        guilds
            .set_social_embed_enabled(10, "spotify", false)
            .await?;
        guilds
            .set_social_embed_enabled(20, "instagram", false)
            .await?;
        users.set_enabled(1, "twitter", false).await?;
        users.set_enabled(1, "tiktok", false).await?;

        for (user_id, guild_id, expected) in [
            (1, Some(10), vec!["spotify", "tiktok", "twitter"]),
            (2, Some(10), vec!["spotify", "twitter"]),
            (1, Some(20), vec!["instagram", "tiktok", "twitter"]),
            (1, None, vec!["tiktok", "twitter"]),
            (2, None, vec![]),
        ] {
            assert_eq!(
                users
                    .effective_disabled_platforms(user_id, guild_id)
                    .await?,
                expected
            );
        }

        users.set_enabled(1, "twitter", true).await?;
        assert_eq!(
            users.effective_disabled_platforms(1, Some(10)).await?,
            ["spotify", "tiktok", "twitter"]
        );
        users.set_enabled(1, "twitter", false).await?;
        guilds.set_social_embed_enabled(10, "twitter", true).await?;
        guilds.set_social_embed_enabled(10, "twitter", true).await?;
        assert_eq!(
            users.effective_disabled_platforms(1, Some(10)).await?,
            ["spotify", "tiktok", "twitter"]
        );
        assert_eq!(
            users.effective_disabled_platforms(2, Some(10)).await?,
            ["spotify"]
        );
        assert_eq!(guilds.disabled_social_platforms(20).await?, ["instagram"]);

        guilds.set_social_embed_enabled(10, "spotify", true).await?;
        assert!(guilds.disabled_social_platforms(10).await?.is_empty());

        users.set_enabled(2, "spotify", false).await?;
        users.reset(1, Some("twitter")).await?;
        users.reset(1, Some("twitter")).await?;
        assert_eq!(users.disabled_platforms(1).await?, ["tiktok"]);

        guilds
            .set_social_embed_enabled(10, "twitter", false)
            .await?;
        guilds
            .set_social_embed_enabled(10, "spotify", false)
            .await?;
        guilds.reset_social_embeds(10, Some("twitter")).await?;
        guilds.reset_social_embeds(10, Some("twitter")).await?;
        assert_eq!(guilds.disabled_social_platforms(10).await?, ["spotify"]);
        assert_eq!(users.disabled_platforms(1).await?, ["tiktok"]);

        users.reset(1, None).await?;
        assert!(users.disabled_platforms(1).await?.is_empty());
        assert_eq!(users.disabled_platforms(2).await?, ["spotify"]);
        assert_eq!(
            users.effective_disabled_platforms(1, Some(20)).await?,
            ["instagram"]
        );

        guilds.reset_social_embeds(10, None).await?;
        assert!(guilds.disabled_social_platforms(10).await?.is_empty());
        assert_eq!(guilds.disabled_social_platforms(20).await?, ["instagram"]);
        assert_eq!(
            users.effective_disabled_platforms(2, Some(10)).await?,
            ["spotify"]
        );
        Ok(())
    }

    #[sqlx::test(migrations = "./migrations")]
    #[ignore = "requires DATABASE_URL with permission to create test databases"]
    async fn opt_outs_persist_and_are_isolated_by_user_and_platform(pool: sqlx::MySqlPool) {
        let database = Database { pool };
        let repository = UserSocialEmbedsRepository::new(&database);
        assert!(repository.disabled_platforms(1).await.unwrap().is_empty());

        repository.set_enabled(1, "twitter", false).await.unwrap();
        repository.set_enabled(1, "twitter", false).await.unwrap();
        repository.set_enabled(1, "spotify", false).await.unwrap();
        repository.set_enabled(2, "tiktok", false).await.unwrap();

        let repository = UserSocialEmbedsRepository::new(&database);
        assert_eq!(
            repository.disabled_platforms(1).await.unwrap(),
            ["spotify", "twitter"]
        );
        assert_eq!(repository.disabled_platforms(2).await.unwrap(), ["tiktok"]);
        assert!(repository.disabled_platforms(3).await.unwrap().is_empty());

        repository.set_enabled(1, "twitter", true).await.unwrap();
        repository.set_enabled(1, "twitter", true).await.unwrap();
        assert_eq!(repository.disabled_platforms(1).await.unwrap(), ["spotify"]);
        assert_eq!(repository.disabled_platforms(2).await.unwrap(), ["tiktok"]);

        repository.set_enabled(1, "spotify", true).await.unwrap();
        assert!(repository.disabled_platforms(1).await.unwrap().is_empty());
    }
}

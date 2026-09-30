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
}

#[cfg(test)]
mod tests {
    use super::*;

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

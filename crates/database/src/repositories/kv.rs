use crate::{Database, DatabaseError};

pub struct KvRepository<'a> {
    database: &'a Database,
}

impl<'a> KvRepository<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        Ok(self
            .get_many(&[key.to_owned()])
            .await?
            .pop()
            .map(|(_, value)| value))
    }

    pub async fn get_many(&self, keys: &[String]) -> Result<Vec<(String, String)>, DatabaseError> {
        let sql = format!(
            "SELECT `key`, value FROM kv WHERE `key` IN ({}) \
             AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP(3))",
            vec!["?"; keys.len()].join(", ")
        );
        let mut query = sqlx::query_as(&sql);
        for key in keys {
            query = query.bind(key);
        }
        Ok(query.fetch_all(self.database.pool()).await?)
    }

    pub async fn set(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u32>,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            "INSERT INTO kv (`key`, value, expires_at) \
             VALUES (?, ?, TIMESTAMPADD(SECOND, ?, CURRENT_TIMESTAMP(3))) \
             ON DUPLICATE KEY UPDATE value = VALUES(value), expires_at = VALUES(expires_at)",
        )
        .bind(key)
        .bind(value)
        .bind(ttl_seconds)
        .execute(self.database.pool())
        .await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), DatabaseError> {
        sqlx::query("DELETE FROM kv WHERE `key` = ?")
            .bind(key)
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
    async fn values_persist_until_they_expire_or_are_deleted(pool: sqlx::MySqlPool) {
        let database = Database { pool };
        let kv = KvRepository::new(&database);

        kv.set("kept", "a", None).await.unwrap();
        kv.set("kept", "b", None).await.unwrap();
        kv.set("cached", "c", Some(60)).await.unwrap();
        kv.set("expired", "d", Some(0)).await.unwrap();
        kv.set("deleted", "e", None).await.unwrap();
        kv.delete("deleted").await.unwrap();

        assert_eq!(kv.get("kept").await.unwrap().as_deref(), Some("b"));
        assert_eq!(kv.get("cached").await.unwrap().as_deref(), Some("c"));
        assert_eq!(kv.get("expired").await.unwrap(), None);
        assert_eq!(kv.get("deleted").await.unwrap(), None);
        let mut found = kv
            .get_many(&["cached".to_owned(), "kept".to_owned(), "missing".to_owned()])
            .await
            .unwrap();
        found.sort();
        assert_eq!(
            found,
            [
                ("cached".to_owned(), "c".to_owned()),
                ("kept".to_owned(), "b".to_owned())
            ]
        );
    }
}

//! Requires DATABASE_URL with permission to create isolated test databases.
use database::{Database, DatabaseConfig, GuildDirectoryRepository};
use feature_guilds::{Freshness, fresh_user_guild, user_guilds};
use sqlx::{ConnectOptions, MySqlPool};

async fn database(pool: &MySqlPool) -> Database {
    // sqlx creates an isolated database for each test; use its URL for the public
    // Database constructor so tests exercise the same repository path as the API.
    let url = pool.connect_options();
    let options = url.as_ref().clone();
    Database::connect(DatabaseConfig {
        url: options.to_url_lossy().to_string(),
        max_connections: 2,
        min_connections: 0,
        connect_timeout_seconds: 5,
    })
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../database/migrations")]
#[ignore = "requires DATABASE_URL with CREATE DATABASE permission"]
async fn cached_reads_skip_discord_but_writes_and_expired_reads_do_not(pool: MySqlPool) {
    let database = database(&pool).await;
    sqlx::query("INSERT INTO `user` (id, name, email, emailVerified, updatedAt) VALUES ('fixture', 'Fixture', 'fixture@example.com', FALSE, CURRENT_TIMESTAMP(3))")
        .execute(&pool).await.unwrap();
    let repository = GuildDirectoryRepository::new(&database);
    let snapshot = r#"[{"id":"9007199254740993","name":"Fixture","icon":null,"permissions":"32"}]"#;
    repository
        .cache_user_guilds("fixture", snapshot, 300)
        .await
        .unwrap();
    let http = reqwest::Client::new();

    // No OAuth account exists: any accidental upstream lookup fails this test.
    let guilds = user_guilds(&database, &http, "fixture", Freshness::Cached)
        .await
        .unwrap();
    assert_eq!(guilds[0].id, "9007199254740993");
    assert!(
        fresh_user_guild(&database, &http, "fixture", 9007199254740993)
            .await
            .is_err()
    );
    // A targeted write lookup must not replace or extend a complete snapshot.
    assert_eq!(
        repository
            .cached_user_guilds("fixture")
            .await
            .unwrap()
            .as_deref(),
        Some(snapshot)
    );
    assert!(
        user_guilds(&database, &http, "fixture", Freshness::Fresh)
            .await
            .is_err()
    );
    assert!(
        repository
            .cached_user_guilds("another-user")
            .await
            .unwrap()
            .is_none()
    );

    sqlx::query("UPDATE user_guild_cache SET expires_at = TIMESTAMPADD(SECOND, -1, CURRENT_TIMESTAMP(3)) WHERE user_id = 'fixture'")
        .execute(&pool).await.unwrap();
    assert!(
        user_guilds(&database, &http, "fixture", Freshness::Cached)
            .await
            .is_err()
    );

    repository
        .cache_user_guilds("fixture", "[]", 300)
        .await
        .unwrap();
    assert!(
        user_guilds(&database, &http, "fixture", Freshness::Cached)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrations = "../database/migrations")]
#[ignore = "requires DATABASE_URL with CREATE DATABASE permission"]
async fn reconciles_bot_membership_per_shard_and_updates_metadata(pool: MySqlPool) {
    let database = database(&pool).await;
    let repository = GuildDirectoryRepository::new(&database);
    repository
        .upsert_bot_guild(1, "Old name", Some("old"), 0)
        .await
        .unwrap();
    repository
        .upsert_bot_guild(2, "Departed offline", None, 0)
        .await
        .unwrap();
    repository
        .upsert_bot_guild(3, "Other shard", None, 1)
        .await
        .unwrap();
    repository.reconcile_shard(0, &[1, 4]).await.unwrap();
    repository
        .upsert_bot_guild(1, "New name", None, 0)
        .await
        .unwrap();
    let installed = repository.installed().await.unwrap();
    assert_eq!(installed.len(), 3);
    assert!(installed.iter().any(|g| g.guild_id == 3));
    assert!(installed.iter().any(|g| g.guild_id == 4));
    let renamed = installed.iter().find(|g| g.guild_id == 1).unwrap();
    assert_eq!(renamed.name.as_deref(), Some("New name"));
    assert!(renamed.icon.is_none());
    assert!(repository.installed_guild(1).await.unwrap().is_some());
    repository.mark_left(1).await.unwrap();
    assert!(repository.installed_guild(1).await.unwrap().is_none());
    assert_eq!(repository.installed().await.unwrap().len(), 2);
    repository.reconcile_shard(0, &[]).await.unwrap();
    assert_eq!(repository.installed().await.unwrap()[0].guild_id, 3);
}

#[sqlx::test(migrations = "../database/migrations")]
#[ignore = "requires DATABASE_URL with CREATE DATABASE permission"]
async fn reconciles_membership_beyond_one_inserts_bind_limit(pool: MySqlPool) {
    let database = database(&pool).await;
    let repository = GuildDirectoryRepository::new(&database);
    let ids: Vec<u64> = (1..=32_768).collect();

    repository.reconcile_shard(0, &ids).await.unwrap();
    let mut installed: Vec<_> = repository
        .installed()
        .await
        .unwrap()
        .into_iter()
        .map(|guild| guild.guild_id)
        .collect();
    installed.sort_unstable();
    assert_eq!(installed, ids);
}

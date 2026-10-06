use super::*;
use axum::body::{Body, to_bytes};
use database::repositories::admin::{RuntimeSnapshot, ShardSnapshot};
use database::{Database, DatabaseConfig, GuildDirectoryRepository};
use sqlx::{ConnectOptions, MySqlPool};
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn runtime_liveness_does_not_misrepresent_stale_or_stopped_processes() {
    assert_eq!(runtime_status(false, 0), "reporting");
    assert_eq!(runtime_status(false, 44), "reporting");
    assert_eq!(runtime_status(false, 45), "stale");
    assert_eq!(runtime_status(false, 3600), "stale");
    assert_eq!(runtime_status(true, 0), "stopped");
    assert_eq!(runtime_status(true, 3600), "stopped");
}

async fn database(pool: &MySqlPool) -> Database {
    Database::connect(DatabaseConfig {
        url: pool.connect_options().to_url_lossy().to_string(),
    })
    .await
    .unwrap()
}

async fn insert_account(pool: &MySqlPool, user: &str, provider: &str, account: &str) {
    sqlx::query("INSERT INTO `user` (id, name, email, emailVerified, updatedAt) VALUES (?, 'Fixture', ?, TRUE, CURRENT_TIMESTAMP(3))")
        .bind(user).bind(format!("{user}@example.com")).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO account (id, userId, providerId, accountId, updatedAt) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP(3))")
        .bind(user).bind(user).bind(provider).bind(account).execute(pool).await.unwrap();
}

async fn session(headers: axum::http::HeaderMap) -> Json<Option<AuthSession>> {
    let user_id = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok());
    Json(user_id.map(|id| serde_json::from_value(serde_json::json!({
        "user": { "id": id, "name": "Fixture", "email": "fixture@example.com", "emailVerified": true,
            "createdAt": "", "updatedAt": "" },
        "session": { "id": "session", "userId": id, "token": "secret-session-token",
            "expiresAt": "", "createdAt": "", "updatedAt": "" }
    })).unwrap()))
}

#[sqlx::test(migrations = "../database/migrations")]
#[ignore = "requires DATABASE_URL with CREATE DATABASE permission"]
async fn every_admin_endpoint_checks_the_linked_discord_account(pool: MySqlPool) {
    let owner = "307952129958477824";
    insert_account(&pool, "owner-internal-id", "discord", owner).await;
    insert_account(&pool, "ordinary-user", "discord", "123").await;
    insert_account(&pool, owner, "discord", "456").await;
    insert_account(&pool, "wrong-provider", "github", owner).await;

    let db = Arc::new(database(&pool).await);
    let directory = GuildDirectoryRepository::new(&db);
    directory
        .upsert_bot_guild(9007199254740993, "Not in owner's guilds", None, 2)
        .await
        .unwrap();
    directory
        .upsert_bot_guild(2, "Departed", None, 0)
        .await
        .unwrap();
    directory.mark_left(2).await.unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let auth_server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route("/api/auth/get-session", get(session)),
        )
        .await
        .unwrap();
    });
    let app = crate::app(AppState {
        database: db,
        auth: auth::AuthClient::new(&format!("http://{address}")).unwrap(),
        http: reqwest::Client::new(),
    });
    for path in ["/admin/access", "/admin/guilds", "/admin/state"] {
        for (user, expected) in [
            (None, StatusCode::UNAUTHORIZED),
            (Some("ordinary-user"), StatusCode::FORBIDDEN),
            (Some(owner), StatusCode::FORBIDDEN),
            (Some("wrong-provider"), StatusCode::FORBIDDEN),
            (Some("unlinked-user"), StatusCode::FORBIDDEN),
            (Some("owner-internal-id"), StatusCode::OK),
        ] {
            let mut request = Request::builder().uri(path).header("x-discord-id", owner);
            if let Some(user) = user {
                request = request.header(header::COOKIE, user);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), expected, "{path}: {user:?}");
            assert_eq!(
                response.headers()[header::CACHE_CONTROL],
                "private, no-store"
            );
            let bytes = to_bytes(response.into_body(), 100_000).await.unwrap();
            let body = String::from_utf8(bytes.to_vec()).unwrap();
            assert!(!body.contains("secret-session-token"));
            if expected == StatusCode::FORBIDDEN {
                assert_eq!(body, "permission denied");
            }
            if expected != StatusCode::OK {
                assert!(!body.contains("Not in owner's guilds"));
            } else if path == "/admin/guilds" {
                assert!(body.contains("\"id\":\"9007199254740993\""));
                assert!(body.contains("Not in owner's guilds"));
                assert!(!body.contains("Departed"));
            }
        }
    }

    // The owner can edit settings of guilds outside their own Discord guild list.
    for (method, path, body) in [
        ("GET", "/guilds/9007199254740993/settings", ""),
        (
            "PUT",
            "/guilds/9007199254740993/settings/general",
            r#"{"commandPrefix":"?","translationLanguage":null}"#,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header(header::COOKIE, "owner-internal-id")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{method} {path}");
    }
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/guilds/2/settings")
                .header(header::COOKIE, "owner-internal-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Revoking the account takes effect on the next request, not session expiry.
    sqlx::query("DELETE FROM account WHERE userId = 'owner-internal-id'")
        .execute(&pool)
        .await
        .unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/state")
                .header(header::COOKIE, "owner-internal-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    // Failed authorization lookups fail closed.
    sqlx::query("DROP TABLE account")
        .execute(&pool)
        .await
        .unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/guilds")
                .header(header::COOKIE, "owner-internal-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    auth_server.abort();
}

#[sqlx::test(migrations = "../database/migrations")]
#[ignore = "requires DATABASE_URL with CREATE DATABASE permission"]
async fn runtime_snapshots_round_trip_and_age_out(pool: MySqlPool) {
    let db = database(&pool).await;
    let repository = AdminRepository::new(&db);
    assert!(repository.runtimes().await.unwrap().is_empty());
    let snapshot = RuntimeSnapshot {
        bot_user_id: Some("9007199254740993".into()),
        bot_name: Some("Yin".into()),
        environment: "development".into(),
        version: "0.1.0".into(),
        uptime_seconds: 10,
        shard_count: Some(1),
        shards: vec![ShardSnapshot {
            id: 0,
            stage: "connecting".into(),
            latency_ms: None,
        }],
        cached_guilds: 2,
        unavailable_guilds: 1,
        cached_users: 3,
        cached_channels: 4,
    };
    repository.publish("first-boot", &snapshot).await.unwrap();
    repository.publish("second-boot", &snapshot).await.unwrap();
    repository.publish("second-boot", &snapshot).await.unwrap();
    let records = repository.runtimes().await.unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0].snapshot.0.bot_user_id.as_deref(),
        Some("9007199254740993")
    );
    assert!(records[0].snapshot.0.shards[0].latency_ms.is_none());
    repository.mark_stopped("first-boot").await.unwrap();
    sqlx::query("UPDATE bot_runtime SET updated_at = TIMESTAMPADD(SECOND, -46, CURRENT_TIMESTAMP(3)) WHERE instance_id = 'second-boot'")
        .execute(&pool).await.unwrap();
    let responses: Vec<RuntimeResponse> = repository
        .runtimes()
        .await
        .unwrap()
        .into_iter()
        .map(Into::into)
        .collect();
    assert_eq!(
        responses
            .iter()
            .find(|r| r.instance_id == "first-boot")
            .unwrap()
            .status,
        "stopped"
    );
    assert_eq!(
        responses
            .iter()
            .find(|r| r.instance_id == "second-boot")
            .unwrap()
            .status,
        "stale"
    );
    sqlx::query(
        "UPDATE bot_runtime SET updated_at = TIMESTAMPADD(HOUR, -25, CURRENT_TIMESTAMP(3))",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert!(repository.runtimes().await.unwrap().is_empty());
    repository.prune_runtimes().await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bot_runtime")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use database::UserSocialEmbedsRepository;
use serde_json::{Value, json};
use sqlx::MySqlPool;
use tower::ServiceExt;

use crate::admin::tests::{database, insert_account, test_app};

async fn send(app: &axum::Router, method: &str, user: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(if method == "GET" {
                    "/me/settings"
                } else {
                    "/me/settings/social"
                })
                .header(header::COOKIE, user)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 100_000).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[sqlx::test(migrations = "../database/migrations")]
#[ignore = "requires DATABASE_URL with CREATE DATABASE permission"]
async fn user_settings_are_keyed_by_the_linked_discord_account(pool: MySqlPool) {
    insert_account(&pool, "internal-id", "discord", "42").await;
    insert_account(&pool, "github-user", "github", "43").await;
    let db = Arc::new(database(&pool).await);
    let (app, auth_server) = test_app(db.clone()).await;
    let platform = |body: &Value, key: &str| {
        body["socialEmbeds"]
            .as_array()
            .unwrap()
            .iter()
            .find(|setting| setting["platform"] == key)
            .unwrap()
            .clone()
    };

    let (status, body) = send(&app, "GET", "internal-id", Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        platform(&body, "twitter"),
        json!({"platform": "twitter", "enabled": true, "display": "full"})
    );

    let (status, body) = send(
        &app,
        "PUT",
        "internal-id",
        json!({"socialEmbeds": [
            {"platform": "twitter", "enabled": true, "display": "media-only"},
            {"platform": "tiktok", "enabled": false, "display": "full"},
        ]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(platform(&body, "twitter")["display"], "media-only");
    assert_eq!(platform(&body, "tiktok")["enabled"], false);
    let repository = UserSocialEmbedsRepository::new(&db);
    assert_eq!(repository.disabled_platforms(42).await.unwrap(), ["tiktok"]);
    assert_eq!(
        repository.displays(42).await.unwrap(),
        [("twitter".to_owned(), "media-only".to_owned())]
    );

    for display in ["full", "unknown"] {
        let (status, _) = send(
            &app,
            "PUT",
            "internal-id",
            json!({"socialEmbeds": [
                {"platform": "tiktok", "enabled": true, "display": "full"},
                {"platform": "twitter", "enabled": true, "display": display},
            ]}),
        )
        .await;
        assert_eq!(
            status,
            if display == "full" {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            }
        );
    }
    assert!(repository.displays(42).await.unwrap().is_empty());
    assert!(repository.disabled_platforms(42).await.unwrap().is_empty());

    for user in ["github-user", "unlinked-user"] {
        let (status, _) = send(&app, "GET", user, Value::Null).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{user}");
    }
    auth_server.abort();
}

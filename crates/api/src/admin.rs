use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use database::repositories::admin::{
    AdminRepository, RUNTIME_STALE_SECONDS, RuntimeRecord, RuntimeSnapshot,
};
use serde::Serialize;

use crate::AppState;
use crate::auth::{self, AuthSession};
use crate::guilds::ApiError;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/access", get(|| async { Json(true) }))
        .route("/guilds", get(guilds))
        .route("/state", get(bot_state))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin))
        .route_layer(middleware::from_fn_with_state(state, auth::require_auth))
        .layer(middleware::from_fn(no_store))
}

async fn no_store(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
}

async fn require_admin(
    State(state): State<AppState>,
    Extension(session): Extension<AuthSession>,
    request: Request,
    next: Next,
) -> Response {
    match AdminRepository::new(&state.database)
        .is_admin(&session.user.id)
        .await
    {
        Ok(true) => next.run(request).await,
        Ok(false) => (StatusCode::FORBIDDEN, "permission denied").into_response(),
        Err(error) => ApiError::Database(error).into_response(),
    }
}

async fn guilds(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(AdminRepository::new(&state.database).guilds().await?))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeResponse {
    instance_id: String,
    snapshot: RuntimeSnapshot,
    status: &'static str,
    age_seconds: i64,
}

impl From<RuntimeRecord> for RuntimeResponse {
    fn from(record: RuntimeRecord) -> Self {
        let status = runtime_status(record.stopped, record.age_seconds);
        Self {
            instance_id: record.instance_id,
            snapshot: record.snapshot.0,
            status,
            age_seconds: record.age_seconds.max(0),
        }
    }
}

fn runtime_status(stopped: bool, age_seconds: i64) -> &'static str {
    if stopped {
        "stopped"
    } else if age_seconds >= RUNTIME_STALE_SECONDS {
        "stale"
    } else {
        // This is process liveness, NOT gateway health. Shards report separately.
        "reporting"
    }
}

async fn bot_state(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let instances: Vec<RuntimeResponse> = AdminRepository::new(&state.database)
        .runtimes()
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(instances))
}

#[cfg(test)]
mod tests;

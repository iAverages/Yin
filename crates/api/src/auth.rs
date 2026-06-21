use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::http::header::{AUTHORIZATION, COOKIE, HeaderMap};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Clone)]
pub struct AuthClient {
    client: reqwest::Client,
    get_session_url: Url,
}

impl AuthClient {
    pub fn new(auth_service_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = auth_service_url.trim_end_matches('/');
        let get_session_url = Url::parse(&format!("{base_url}/api/auth/get-session"))?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;

        Ok(Self {
            client,
            get_session_url,
        })
    }

    pub async fn get_session(
        &self,
        headers: &HeaderMap,
    ) -> Result<Option<AuthSession>, AuthClientError> {
        let mut request = self.client.get(self.get_session_url.clone());

        if let Some(cookie) = headers.get(COOKIE) {
            request = request.header(COOKIE, cookie.clone());
        }

        if let Some(authorization) = headers.get(AUTHORIZATION) {
            request = request.header(AUTHORIZATION, authorization.clone());
        }

        let response = request.send().await.map_err(AuthClientError::Request)?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AuthClientError::Status { status, body });
        }

        response.json().await.map_err(AuthClientError::Decode)
    }
}

#[derive(Debug)]
pub enum AuthClientError {
    Request(reqwest::Error),
    Decode(reqwest::Error),
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
}

impl std::fmt::Display for AuthClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(error) => write!(f, "auth service request failed: {error}"),
            Self::Decode(error) => write!(f, "auth service response decode failed: {error}"),
            Self::Status { status, body } => {
                write!(f, "auth service returned {status}: {body}")
            }
        }
    }
}

impl std::error::Error for AuthClientError {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthSession {
    pub user: AuthUser,
    pub session: Session,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub async fn require_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: HeaderMap,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    match state.auth.get_session(&headers).await {
        Ok(Some(session)) => {
            request.extensions_mut().insert(session);
            next.run(request).await
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "unauthorized").into_response(),
        Err(error) => {
            tracing::error!(error = %error, "failed to validate auth session");
            (StatusCode::SERVICE_UNAVAILABLE, "auth service unavailable").into_response()
        }
    }
}

pub async fn current_user(Extension(session): Extension<AuthSession>) -> Json<AuthUser> {
    Json(session.user)
}

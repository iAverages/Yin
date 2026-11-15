use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::http::header::{AUTHORIZATION, COOKIE, HeaderMap};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Clone)]
pub struct AuthClient {
    client: reqwest::Client,
    get_session_url: Url,
    discord_token_url: Url,
}

impl AuthClient {
    pub fn new(auth_service_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = auth_service_url.trim_end_matches('/');
        let get_session_url = Url::parse(&format!("{base_url}/api/auth/get-session"))?;
        let discord_token_url = Url::parse(&format!("{base_url}/discord/access-token"))?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()?;

        Ok(Self {
            client,
            get_session_url,
            discord_token_url,
        })
    }

    pub async fn get_session(
        &self,
        headers: &HeaderMap,
    ) -> Result<Option<AuthSession>, AuthClientError> {
        let credentials: HeaderMap = [COOKIE, AUTHORIZATION]
            .into_iter()
            .filter_map(|name| headers.get(&name).cloned().map(|value| (name, value)))
            .collect();

        let response = self
            .client
            .get(self.get_session_url.clone())
            .headers(credentials.clone())
            .send()
            .await
            .map_err(AuthClientError::Request)?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AuthClientError::Status { status, body });
        }

        let session: Option<AuthSession> =
            response.json().await.map_err(AuthClientError::Decode)?;
        Ok(session.map(|session| AuthSession {
            credentials,
            ..session
        }))
    }

    pub async fn discord_token(
        &self,
        credentials: &HeaderMap,
    ) -> Result<Option<String>, AuthClientError> {
        let response = self
            .client
            .get(self.discord_token_url.clone())
            .headers(credentials.clone())
            .send()
            .await
            .map_err(AuthClientError::Request)?;
        let status = response.status();

        if matches!(status, StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED) {
            return Ok(None);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AuthClientError::Status { status, body });
        }

        let token: DiscordToken = response.json().await.map_err(AuthClientError::Decode)?;

        Ok(Some(token.access_token).filter(|token| !token.is_empty()))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscordToken {
    access_token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthClientError {
    #[error("auth service request failed: {0}")]
    Request(reqwest::Error),
    #[error("auth service response decode failed: {0}")]
    Decode(reqwest::Error),
    #[error("auth service returned {status}: {body}")]
    Status {
        status: reqwest::StatusCode,
        body: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthSession {
    pub user: AuthUser,
    pub session: Session,
    /// The caller's own auth headers, kept to fetch their Discord token later.
    #[serde(skip)]
    pub credentials: HeaderMap,
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

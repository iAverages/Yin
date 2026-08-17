mod admin;
mod auth;
mod config;
mod guilds;

use std::sync::Arc;

use axum::Router;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::middleware;
use axum::routing::{delete, get, post, put};
use tower_http::cors::CorsLayer;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<database::Database>,
    pub auth: auth::AuthClient,
    pub http: reqwest::Client,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = config::ApiConfig::from_env()?;
    let database =
        Arc::new(database::Database::connect(database::DatabaseConfig::from_env()?).await?);
    let auth = auth::AuthClient::new(&config.auth_service_url)?;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()?;
    let cors = cors_layer(&config.cors_allowed_origins)?;
    let state = AppState {
        database,
        auth,
        http,
    };
    let app = app(state).layer(cors);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;

    tracing::info!(address = %config.bind_addr, "api listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn cors_layer(origins: &[String]) -> Result<CorsLayer, Error> {
    let origins = origins
        .iter()
        .map(|origin| origin.parse::<HeaderValue>())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]))
}

fn app(state: AppState) -> Router {
    let protected = Router::new()
        .route("/auth/user", get(auth::current_user))
        .route("/guilds", get(guilds::list_managed_guilds))
        .route(
            "/guilds/{guild_id}/settings",
            get(guilds::get_guild_settings),
        )
        .route(
            "/guilds/{guild_id}/settings/general",
            put(guilds::update_general_settings),
        )
        .route(
            "/guilds/{guild_id}/settings/social",
            put(guilds::update_social_embeds),
        )
        .route(
            "/guilds/{guild_id}/custom-commands",
            post(guilds::upsert_custom_command),
        )
        .route(
            "/guilds/{guild_id}/custom-commands/{name}",
            delete(guilds::delete_custom_command),
        )
        .route(
            "/guilds/{guild_id}/ladder-rules",
            post(guilds::create_ladder_rule),
        )
        .route(
            "/guilds/{guild_id}/ladder-rules/{rule_id}",
            put(guilds::update_ladder_rule).delete(guilds::delete_ladder_rule),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    Router::new()
        .route("/health", get(health))
        .merge(protected)
        .nest("/admin", admin::routes(state.clone()))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(error = %error, "failed to install ctrl-c handler");
        }
    };

    #[cfg(unix)]
    {
        let terminate = async {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut signal) => {
                    signal.recv().await;
                }
                Err(error) => tracing::error!(error = %error, "failed to install sigterm handler"),
            }
        };

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }

    #[cfg(not(unix))]
    ctrl_c.await;

    tracing::info!("shutdown signal received");
}

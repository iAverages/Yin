mod auth;
mod config;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::get;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<database::Database>,
    pub auth: auth::AuthClient,
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
    let state = AppState { database, auth };
    let app = app(state);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;

    tracing::info!(address = %config.bind_addr, "api listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn app(state: AppState) -> Router {
    let protected = Router::new()
        .route("/auth/user", get(auth::current_user))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    Router::new()
        .route("/health", get(health))
        .merge(protected)
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

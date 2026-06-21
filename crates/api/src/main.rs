use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<database::Database>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let bind_addr = bind_addr()?;
    let database =
        Arc::new(database::Database::connect(database::DatabaseConfig::from_env()?).await?);
    let app = Router::new().with_state(AppState { database });
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    tracing::info!(address = %bind_addr, "api listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn bind_addr() -> Result<SocketAddr, Error> {
    let value = env::var("API_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_owned());
    Ok(value.parse()?)
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

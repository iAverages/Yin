#[tokio::main]
async fn main() -> Result<(), database::DatabaseError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = database::DatabaseConfig::from_env()?;
    let database = database::Database::connect(config).await?;

    tracing::info!("running database migrations");
    database::run_migrations(database.pool()).await?;
    tracing::info!("database migrations complete");

    Ok(())
}

mod config;
mod diagnostics;
mod framework;
mod shutdown;

use std::sync::Arc;
use std::time::{Duration, Instant};

use database::repositories::admin::AdminRepository;

use bot_core::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let started_at = Instant::now();
    let instance_id = uuid::Uuid::now_v7().to_string();
    let config = config::BotConfig::from_env()?;
    let database = Arc::new(database::Database::connect(config.database).await?);
    let feature_flags = feature_flags::FeatureFlags::from_env().await?;
    let framework = framework::build(
        config.environment,
        config.dev_guild_id,
        database.clone(),
        feature_flags,
    );
    let intents = bot_core::serenity::GatewayIntents::GUILDS
        | bot_core::serenity::GatewayIntents::GUILD_MODERATION
        | bot_core::serenity::GatewayIntents::GUILD_MESSAGES
        | bot_core::serenity::GatewayIntents::DIRECT_MESSAGES
        | bot_core::serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = bot_core::serenity::ClientBuilder::new(config.discord_token, intents)
        .framework(framework)
        .await?;

    let shard_manager = client.shard_manager.clone();

    let diagnostics = diagnostics::run(
        database.clone(),
        shard_manager.clone(),
        client.cache.clone(),
        &instance_id,
        config.environment,
        started_at,
    );
    let result = tokio::select! {
        result = client.start() => result.map_err(Error::from),
        result = shutdown::signal() => {
            if result.is_ok() {
                tracing::info!("shutdown signal received");
            }
            result
        },
        _ = diagnostics => unreachable!("diagnostics runs until shutdown"),
    };

    // Stop publishing before marking the process stopped. An abrupt termination
    // instead ages into a stale snapshot after 45 seconds.
    if !matches!(
        tokio::time::timeout(
            Duration::from_secs(2),
            AdminRepository::new(&database).mark_stopped(&instance_id),
        )
        .await,
        Ok(Ok(()))
    ) {
        tracing::warn!("could not mark bot diagnostics stopped");
    }
    shard_manager.shutdown_all().await;
    result
}

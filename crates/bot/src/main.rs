mod config;
mod framework;
mod shutdown;

use std::sync::Arc;

use bot_core::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = config::BotConfig::from_env()?;
    let database = Arc::new(database::Database::connect(config.database).await?);
    let framework = framework::build(
        config.environment,
        config.dev_guild_id,
        database,
        config.auth_service_url,
        config.auth_internal_token,
    );
    let intents = bot_core::serenity::GatewayIntents::GUILDS
        | bot_core::serenity::GatewayIntents::GUILD_MODERATION
        | bot_core::serenity::GatewayIntents::GUILD_MEMBERS
        | bot_core::serenity::GatewayIntents::GUILD_MESSAGES
        | bot_core::serenity::GatewayIntents::DIRECT_MESSAGES
        | bot_core::serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = bot_core::serenity::ClientBuilder::new(config.discord_token, intents)
        .framework(framework)
        .await?;

    let shard_manager = client.shard_manager.clone();

    tokio::select! {
        result = client.start() => result?,
        result = shutdown::signal() => {
            result?;
            tracing::info!("shutdown signal received");
            shard_manager.shutdown_all().await;
        }
    }

    Ok(())
}

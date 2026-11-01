mod config;
mod diagnostics;
mod events;
mod gateway;
mod shutdown;

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use bot_core::command::Command;
use bot_core::{BotState, Environment, Error};
use database::GuildDirectoryRepository;
use database::repositories::admin::AdminRepository;
use tokio::task::JoinSet;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{CloseFrame, Config, Intents, Shard, ShardId};
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("no other rustls crypto provider is installed");

    let started_at = Instant::now();
    let instance_id = uuid::Uuid::now_v7().to_string();
    let config = config::BotConfig::from_env()?;
    let database = Arc::new(database::Database::connect(config.database).await?);
    let feature_flags = feature_flags::FeatureFlags::from_env().await?;

    let http = Arc::new(twilight_http::Client::new(config.discord_token.clone()));
    let application = http.current_user_application().await?.model().await?;
    let owner_ids = match &application.team {
        Some(team) => team.members.iter().map(|member| member.user.id).collect(),
        None => application.owner.iter().map(|owner| owner.id).collect(),
    };

    let commands: Arc<[Command]> = commands().into();
    register_commands(
        &http,
        application.id,
        &commands,
        config.environment,
        config.dev_guild_id,
    )
    .await?;

    let cache = Arc::new(
        InMemoryCache::builder()
            .resource_types(
                ResourceType::GUILD
                    | ResourceType::CHANNEL
                    | ResourceType::ROLE
                    | ResourceType::MEMBER
                    | ResourceType::USER
                    | ResourceType::USER_CURRENT,
            )
            .build(),
    );
    let state = Arc::new(BotState {
        started_at,
        environment: config.environment,
        panel_url: config.panel_url,
        database: database.clone(),
        feature_flags,
        http: http.clone(),
        cache: cache.clone(),
        owner_ids,
    });

    let intents = Intents::GUILDS
        | Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES
        | Intents::MESSAGE_CONTENT;
    let gateway_config = Config::new(config.discord_token, intents);

    let shard_count = http.gateway().authed().await?.model().await?.shards;
    let shards: Vec<_> = (0..shard_count)
        .map(|id| Shard::with_config(ShardId::new(id, shard_count), gateway_config.clone()))
        .collect();

    let senders: Vec<_> = shards.iter().map(Shard::sender).collect();
    let status = gateway::ShardStatus::default();
    let mut runners = JoinSet::new();

    for shard in shards {
        runners.spawn(gateway::run_shard(
            shard,
            state.clone(),
            commands.clone(),
            status.clone(),
        ));
    }

    let diagnostics = diagnostics::run(
        database.clone(),
        status,
        cache,
        shard_count,
        &instance_id,
        config.environment,
        started_at,
    );

    tokio::spawn(purge_departed_guilds(database.clone()));

    let result = tokio::select! {
        _ = runners.join_next() => Err("a gateway shard stopped unexpectedly".into()),
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

    gateway::SHUTDOWN.store(true, Ordering::Relaxed);

    for sender in senders {
        _ = sender.close(CloseFrame::NORMAL);
    }

    let closed = tokio::time::timeout(Duration::from_secs(5), async {
        while runners.join_next().await.is_some() {}
    })
    .await;

    if closed.is_err() {
        tracing::warn!("gateway shards did not close in time");
    }

    result
}

async fn purge_departed_guilds(database: Arc<database::Database>) {
    let period = Duration::from_secs(60 * 60);
    let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + period, period);

    loop {
        interval.tick().await;
        match GuildDirectoryRepository::new(&database)
            .purge_departed()
            .await
        {
            Ok(0) => {}
            Ok(rows) => tracing::info!(rows, "purged data for departed guilds"),
            Err(error) => tracing::warn!(%error, "failed to purge departed guilds"),
        }
    }
}

fn commands() -> Vec<Command> {
    let mut commands = feature_endfield::commands();
    commands.extend(feature_info::commands());
    commands.extend(feature_settings::commands());
    commands.extend(feature_social::commands());
    commands
}

async fn register_commands(
    http: &twilight_http::Client,
    application_id: Id<ApplicationMarker>,
    commands: &[Command],
    environment: Environment,
    dev_guild_id: Option<Id<GuildMarker>>,
) -> Result<(), Error> {
    let definitions: Vec<_> = commands
        .iter()
        .map(|command| command.definition.clone())
        .collect();

    let interaction = http.interaction(application_id);

    match environment {
        Environment::Development => {
            let guild_id = dev_guild_id.expect("development config requires a dev guild id");
            interaction
                .set_guild_commands(guild_id, &definitions)
                .await?;
            tracing::info!(guild_id = %guild_id, "registered guild commands");
        }
        Environment::Production => {
            interaction.set_global_commands(&definitions).await?;
            tracing::info!("registered global commands");
        }
    }
    Ok(())
}

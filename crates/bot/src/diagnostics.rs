use std::sync::Arc;
use std::time::{Duration, Instant};

use bot_core::Environment;
use bot_core::serenity::{Cache, ShardManager};
use database::repositories::admin::{AdminRepository, RuntimeSnapshot, ShardSnapshot};

/// Runs independently of Poise setup, including while the gateway is connecting.
/// Only the short in-memory copy holds the runner lock; DB IO never does.
pub async fn run(
    database: Arc<database::Database>,
    manager: Arc<ShardManager>,
    cache: Arc<Cache>,
    instance_id: &str,
    environment: Environment,
    started_at: Instant,
) {
    let repository = AdminRepository::new(&database);
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        let mut shards: Vec<_> = {
            let runners = manager.runners.lock().await;
            runners
                .iter()
                .map(|(id, runner)| ShardSnapshot {
                    id: id.0,
                    stage: runner.stage.to_string(),
                    latency_ms: runner.latency.map(|latency| latency.as_millis() as u64),
                })
                .collect()
        };
        shards.sort_by_key(|shard| shard.id);
        let (bot_user_id, bot_name) = {
            let user = cache.current_user();
            if user.name.is_empty() {
                (None, None)
            } else {
                (Some(user.id.to_string()), Some(user.name.clone()))
            }
        };
        let snapshot = RuntimeSnapshot {
            shard_count: bot_user_id.as_ref().map(|_| cache.shard_count()),
            bot_user_id,
            bot_name,
            environment: environment.to_string(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            uptime_seconds: started_at.elapsed().as_secs(),
            shards,
            cached_guilds: cache.guild_count(),
            unavailable_guilds: cache.unavailable_guilds().len(),
            cached_users: cache.user_count(),
            cached_channels: cache.guild_channel_count(),
        };
        // Diagnostics must not prevent the gateway from starting/reconnecting.
        let result = tokio::time::timeout(Duration::from_secs(5), async {
            repository.publish(instance_id, &snapshot).await?;
            repository.prune_runtimes().await
        })
        .await;
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => tracing::warn!(%error, "failed to publish bot diagnostics"),
            Err(_) => tracing::warn!("bot diagnostics publish timed out"),
        }
    }
}

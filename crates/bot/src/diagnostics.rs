use std::sync::Arc;
use std::time::{Duration, Instant};

use bot_core::Environment;
use database::repositories::admin::{AdminRepository, RuntimeSnapshot};
use twilight_cache_inmemory::InMemoryCache;

use crate::gateway::ShardStatus;

pub async fn run(
    database: Arc<database::Database>,
    status: ShardStatus,
    cache: Arc<InMemoryCache>,
    shard_count: u32,
    instance_id: &str,
    environment: Environment,
    started_at: Instant,
) {
    let repository = AdminRepository::new(&database);
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let shards: Vec<_> = status
            .lock()
            .expect("shard status lock poisoned")
            .values()
            .cloned()
            .collect();
        let (bot_user_id, bot_name) = match cache.current_user() {
            Some(user) => (Some(user.id.to_string()), Some(user.name)),
            None => (None, None),
        };

        let stats = cache.stats();

        let snapshot = RuntimeSnapshot {
            shard_count: bot_user_id.as_ref().map(|_| shard_count),
            bot_user_id,
            bot_name,
            environment: environment.to_string(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            uptime_seconds: started_at.elapsed().as_secs(),
            shards,
            cached_guilds: stats.guilds(),
            unavailable_guilds: stats.unavailable_guilds(),
            cached_users: stats.users(),
            cached_channels: stats.channels(),
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

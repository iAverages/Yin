use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bot_core::BotState;
use bot_core::command::Command;
use database::repositories::admin::ShardSnapshot;
use twilight_gateway::{Event, EventTypeFlags, Shard, ShardState, StreamExt};

use crate::events;

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub type ShardStatus = Arc<Mutex<BTreeMap<u32, ShardSnapshot>>>;

pub async fn run_shard(
    mut shard: Shard,
    state: Arc<BotState>,
    commands: Arc<[Command]>,
    status: ShardStatus,
) {
    let shard_id = shard.id().number();
    record_status(&status, &shard);

    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        record_status(&status, &shard);

        let event = match item {
            Ok(event) => event,
            Err(error) => {
                tracing::warn!(shard_id, error = %error, "failed to receive gateway event");
                continue;
            }
        };

        if matches!(event, Event::GatewayClose(_)) && SHUTDOWN.load(Ordering::Relaxed) {
            break;
        }

        state.cache.update(&event);

        let kind = event.kind();
        let state = state.clone();
        let commands = commands.clone();

        tokio::spawn(async move {
            if let Err(error) = events::handle(state, commands, shard_id, event).await {
                tracing::error!(
                    event = kind.name(),
                    error = %events::format_error_chain(error.as_ref()),
                    "event handler failed"
                );
            }
        });
    }
}

fn record_status(status: &ShardStatus, shard: &Shard) {
    let snapshot = ShardSnapshot {
        id: shard.id().number(),
        stage: stage(shard.state()).to_owned(),
        latency_ms: shard
            .latency()
            .recent()
            .first()
            .map(|latency| latency.as_millis() as u64),
    };
    status
        .lock()
        .expect("shard status lock poisoned")
        .insert(snapshot.id, snapshot);
}

fn stage(state: ShardState) -> &'static str {
    match state {
        ShardState::Active => "connected",
        ShardState::Disconnected { .. } => "disconnected",
        ShardState::FatallyClosed => "closed",
        ShardState::Identifying => "identifying",
        ShardState::Resuming => "resuming",
    }
}

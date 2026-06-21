use uuid::Uuid;

use crate::Context;

#[derive(Debug, Clone, Copy)]
pub struct CommandTrace {
    pub trace_id: Uuid,
}

impl CommandTrace {
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::now_v7(),
        }
    }
}

impl Default for CommandTrace {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn initialize(ctx: Context<'_>) -> CommandTrace {
    let trace = CommandTrace::new();
    ctx.set_invocation_data(trace).await;
    tracing::info!(trace_id = %trace.trace_id, command = %ctx.invocation_string(), "command started");
    trace
}

pub async fn current(ctx: Context<'_>) -> CommandTrace {
    let trace = ctx.invocation_data::<CommandTrace>().await;
    trace.as_deref().copied().unwrap_or_else(CommandTrace::new)
}

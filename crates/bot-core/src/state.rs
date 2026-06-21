use std::time::Instant;

use crate::Environment;

pub struct BotState {
    pub started_at: Instant,
    pub environment: Environment,
}

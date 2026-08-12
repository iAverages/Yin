use std::sync::Arc;
use std::time::Instant;

use crate::Environment;

pub struct BotState {
    pub started_at: Instant,
    pub environment: Environment,
    pub database: Arc<database::Database>,
    pub auth_service_url: String,
    pub auth_internal_token: Option<String>,
}

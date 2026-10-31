use std::sync::Arc;
use std::time::Instant;

use crate::Environment;

pub struct BotState {
    pub started_at: Instant,
    pub environment: Environment,
    pub panel_url: Option<String>,
    pub database: Arc<database::Database>,
    pub feature_flags: feature_flags::FeatureFlags,
}

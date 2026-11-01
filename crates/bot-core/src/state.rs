use std::sync::Arc;
use std::time::Instant;

use twilight_cache_inmemory::InMemoryCache;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;

use crate::Environment;

pub struct BotState {
    pub started_at: Instant,
    pub environment: Environment,
    pub panel_url: Option<String>,
    pub database: Arc<database::Database>,
    pub feature_flags: feature_flags::FeatureFlags,
    pub http: Arc<twilight_http::Client>,
    pub cache: Arc<InMemoryCache>,
    pub owner_ids: Vec<Id<UserMarker>>,
}

pub mod config;
pub mod error;
pub mod repositories;

use std::time::Duration;

use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, MySqlPool};

pub use config::DatabaseConfig;
pub use error::DatabaseError;
pub use repositories::guild_settings::{GuildSettings, GuildSettingsRepository};
pub use repositories::moderation::{
    ChannelLockOperation, ChannelLockTarget, ExternalAuditCaseInsert, ModerationCase,
    ModerationCaseHistory, ModerationRepository, ModerationStatus, NewChannelLockOperation,
    NewChannelLockTarget, NewExternalAuditCase, NewModerationCase, NewPunishmentLadderExecution,
    NewPunishmentLadderRule, NewWarn, NewWarning, PunishmentLadderExecution, PunishmentLadderRule,
    WarnResult, Warning,
};

pub struct Database {
    pool: MySqlPool,
}

impl Database {
    pub async fn connect(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        let pool = MySqlPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .connect(&config.url)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }
}

pub async fn run_migrations(pool: &MySqlPool) -> Result<(), DatabaseError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(DatabaseError::Migration)
}

pub type Query = sea_query::SelectStatement;
pub type MysqlQueryBuilder = sea_query::MysqlQueryBuilder;
pub type Executor<'a> = &'a MySqlPool;
pub type Transaction<'a> = sqlx::Transaction<'a, MySql>;

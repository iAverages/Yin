#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("{name} is required")]
    MissingEnv { name: &'static str },

    #[error("database error")]
    Sqlx(#[from] sqlx::Error),

    #[error("database migration error")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("channel {channel_id} already belongs to an active lock operation")]
    ChannelAlreadyLocked { channel_id: u64 },
}

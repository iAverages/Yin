#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("{name} is required")]
    MissingEnv { name: &'static str },

    #[error("failed to read {name}")]
    Env {
        name: &'static str,
        #[source]
        source: std::env::VarError,
    },

    #[error("{name} must be an integer")]
    InvalidInteger {
        name: &'static str,
        #[source]
        source: std::num::ParseIntError,
    },

    #[error("database error")]
    Sqlx(#[from] sqlx::Error),

    #[error("database migration error")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

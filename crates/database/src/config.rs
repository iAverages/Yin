use std::env;

use crate::DatabaseError;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, DatabaseError> {
        let url = env::var("DATABASE_URL").map_err(|_| DatabaseError::MissingEnv {
            name: "DATABASE_URL",
        })?;

        Ok(Self { url })
    }
}

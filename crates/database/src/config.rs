use std::env;

use crate::DatabaseError;

const DEFAULT_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_MIN_CONNECTIONS: u32 = 0;
const DEFAULT_CONNECT_TIMEOUT_SECONDS: u64 = 10;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, DatabaseError> {
        let url = env::var("DATABASE_URL").map_err(|_| DatabaseError::MissingEnv {
            name: "DATABASE_URL",
        })?;

        let max_connections = parse_optional_env(
            "DATABASE_MAX_CONNECTIONS",
            DEFAULT_MAX_CONNECTIONS,
            parse_u32,
        )?;
        let min_connections = parse_optional_env(
            "DATABASE_MIN_CONNECTIONS",
            DEFAULT_MIN_CONNECTIONS,
            parse_u32,
        )?;
        let connect_timeout_seconds = parse_optional_env(
            "DATABASE_CONNECT_TIMEOUT_SECONDS",
            DEFAULT_CONNECT_TIMEOUT_SECONDS,
            parse_u64,
        )?;

        Ok(Self {
            url,
            max_connections,
            min_connections,
            connect_timeout_seconds,
        })
    }
}

fn parse_optional_env<T>(
    name: &'static str,
    default: T,
    parse: impl FnOnce(String, &'static str) -> Result<T, DatabaseError>,
) -> Result<T, DatabaseError> {
    match env::var(name) {
        Ok(value) => parse(value, name),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(DatabaseError::Env {
            name,
            source: error,
        }),
    }
}

fn parse_u32(value: String, name: &'static str) -> Result<u32, DatabaseError> {
    value
        .parse()
        .map_err(|source| DatabaseError::InvalidInteger { name, source })
}

fn parse_u64(value: String, name: &'static str) -> Result<u64, DatabaseError> {
    value
        .parse()
        .map_err(|source| DatabaseError::InvalidInteger { name, source })
}

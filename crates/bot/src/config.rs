use std::env;
use std::str::FromStr;

use bot_core::serenity;
use bot_core::{Environment, Error};

pub struct BotConfig {
    pub discord_token: String,
    pub environment: Environment,
    pub dev_guild_id: Option<serenity::GuildId>,
    pub database: database::DatabaseConfig,
    pub auth_service_url: String,
    pub auth_internal_token: Option<String>,
}

impl BotConfig {
    pub fn from_env() -> Result<Self, Error> {
        let discord_token =
            env::var("DISCORD_TOKEN").map_err(|_| "DISCORD_TOKEN is required to start the bot")?;

        let environment = match env::var("APP_ENV") {
            Ok(value) => Environment::from_str(&value)?,
            Err(env::VarError::NotPresent) => Environment::default(),
            Err(error) => return Err(Box::new(error)),
        };

        let dev_guild_id = match env::var("DISCORD_DEV_GUILD_ID") {
            Ok(value) => Some(serenity::GuildId::new(value.parse::<u64>().map_err(
                |_| "DISCORD_DEV_GUILD_ID must be a valid Discord guild snowflake",
            )?)),
            Err(env::VarError::NotPresent) => None,
            Err(error) => return Err(Box::new(error)),
        };

        if environment == Environment::Development && dev_guild_id.is_none() {
            return Err("DISCORD_DEV_GUILD_ID is required when APP_ENV is development".into());
        }

        let database = database::DatabaseConfig::from_env()?;
        let auth_service_url =
            env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://auth:3001".to_owned());
        let auth_internal_token = match env::var("AUTH_INTERNAL_TOKEN") {
            Ok(value) if !value.trim().is_empty() => Some(value),
            Ok(_) | Err(env::VarError::NotPresent) => None,
            Err(error) => return Err(Box::new(error)),
        };

        Ok(Self {
            discord_token,
            environment,
            dev_guild_id,
            database,
            auth_service_url,
            auth_internal_token,
        })
    }
}

use std::collections::HashMap;

use bot_core::serenity;
use database::{BotGuild, Database, GuildDirectoryRepository};
use serde::{Deserialize, Serialize};

pub const USER_GUILD_CACHE_TTL_SECONDS: u32 = 300;
const DISCORD_GUILDS_URL: &str = "https://discord.com/api/v10/users/@me/guilds";

#[derive(Debug, thiserror::Error)]
pub enum GuildError {
    #[error(transparent)]
    Database(#[from] database::DatabaseError),
    #[error("Discord access is missing or expired. Sign in with Discord again.")]
    Unauthorized,
    #[error("Discord guild lookup failed. Try again shortly.")]
    Discord(#[from] reqwest::Error),
    #[error("Invalid guild snapshot")]
    Snapshot(#[from] serde_json::Error),
    #[error("Discord returned an invalid guild page")]
    InvalidPage,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Cached,
    Fresh,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserGuild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub permissions: String,
    #[serde(default)]
    pub owner: bool,
}

impl UserGuild {
    pub fn can_manage(&self) -> bool {
        self.owner
            || self
                .permissions
                .parse::<u64>()
                .is_ok_and(|bits| bits & (8 | 32) != 0)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedGuild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub permissions: String,
    pub can_manage: bool,
    pub bot_installed: bool,
}

pub fn managed_guilds(guilds: Vec<UserGuild>, installed: Vec<BotGuild>) -> Vec<ManagedGuild> {
    let installed: HashMap<_, _> = installed
        .into_iter()
        .map(|guild| (guild.guild_id.to_string(), guild))
        .collect();
    let mut result: Vec<_> = guilds
        .into_iter()
        .filter(|guild| guild.can_manage())
        .map(|guild| {
            let bot = installed.get(&guild.id);
            ManagedGuild {
                name: bot.and_then(|bot| bot.name.clone()).unwrap_or(guild.name),
                icon: match bot {
                    Some(bot) if bot.name.is_some() => bot.icon.clone(),
                    _ => guild.icon,
                },
                id: guild.id,
                permissions: guild.permissions,
                can_manage: true,
                bot_installed: bot.is_some(),
            }
        })
        .collect();
    result.sort_by(|a, b| {
        b.bot_installed
            .cmp(&a.bot_installed)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.id.cmp(&b.id))
    });
    result
}

pub async fn user_guilds(
    database: &Database,
    http: &reqwest::Client,
    user_id: &str,
    freshness: Freshness,
) -> Result<Vec<UserGuild>, GuildError> {
    let repository = GuildDirectoryRepository::new(database);
    if freshness == Freshness::Cached
        && let Some(snapshot) = repository.cached_user_guilds(user_id).await?
    {
        return Ok(serde_json::from_str(&snapshot)?);
    }
    let token = repository
        .discord_token(user_id)
        .await?
        .ok_or(GuildError::Unauthorized)?;
    let guilds = fetch_user_guilds_at(http, &token, DISCORD_GUILDS_URL).await?;
    repository
        .cache_user_guilds(
            user_id,
            &serde_json::to_string(&guilds)?,
            USER_GUILD_CACHE_TTL_SECONDS,
        )
        .await?;
    Ok(guilds)
}

pub async fn fresh_user_guild(
    database: &Database,
    http: &reqwest::Client,
    user_id: &str,
    guild_id: u64,
) -> Result<Option<UserGuild>, GuildError> {
    let token = GuildDirectoryRepository::new(database)
        .discord_token(user_id)
        .await?
        .ok_or(GuildError::Unauthorized)?;
    fetch_user_guild_at(http, &token, guild_id, DISCORD_GUILDS_URL).await
}

async fn fetch_user_guild_at(
    http: &reqwest::Client,
    token: &str,
    guild_id: u64,
    url: &str,
) -> Result<Option<UserGuild>, GuildError> {
    let after = guild_id.saturating_sub(1).to_string();
    let response = http
        .get(url)
        .bearer_auth(token)
        .query(&[("limit", "1"), ("after", after.as_str())])
        .send()
        .await?;
    if matches!(response.status().as_u16(), 401 | 403) {
        return Err(GuildError::Unauthorized);
    }
    let page: Vec<UserGuild> = response.error_for_status()?.json().await?;
    if page.len() > 1 {
        return Err(GuildError::InvalidPage);
    }
    Ok(page
        .into_iter()
        .find(|guild| guild.id == guild_id.to_string()))
}

async fn fetch_user_guilds_at(
    http: &reqwest::Client,
    token: &str,
    url: &str,
) -> Result<Vec<UserGuild>, GuildError> {
    let mut guilds = Vec::new();
    let mut after = String::from("0");
    loop {
        let response = http
            .get(url)
            .bearer_auth(token)
            .query(&[("limit", "200"), ("after", after.as_str())])
            .send()
            .await?;
        if matches!(response.status().as_u16(), 401 | 403) {
            return Err(GuildError::Unauthorized);
        }
        let page: Vec<UserGuild> = response.error_for_status()?.json().await?;
        let complete = page.len() < 200;
        if let Some(last) = page.last() {
            if last
                .id
                .parse::<u64>()
                .ok()
                .zip(after.parse::<u64>().ok())
                .is_none_or(|(last, previous)| last <= previous)
            {
                return Err(GuildError::InvalidPage);
            }
            after = last.id.clone();
        }
        guilds.extend(page);
        if complete {
            return Ok(guilds);
        }
    }
}

pub async fn handle_event(
    database: &Database,
    shard_id: u32,
    event: &serenity::FullEvent,
) -> Result<(), database::DatabaseError> {
    let repository = GuildDirectoryRepository::new(database);
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            let ids: Vec<_> = data_about_bot
                .guilds
                .iter()
                .map(|guild| guild.id.get())
                .collect();
            repository.reconcile_shard(shard_id, &ids).await?;
        }
        serenity::FullEvent::GuildCreate { guild, .. } => {
            let icon = guild.icon.map(|icon| icon.to_string());
            repository
                .upsert_bot_guild(guild.id.get(), &guild.name, icon.as_deref(), shard_id)
                .await?;
        }
        serenity::FullEvent::GuildUpdate {
            new_data: guild, ..
        } => {
            let icon = guild.icon.map(|icon| icon.to_string());
            repository
                .upsert_bot_guild(guild.id.get(), &guild.name, icon.as_deref(), shard_id)
                .await?;
        }
        serenity::FullEvent::GuildDelete { incomplete, .. } if !incomplete.unavailable => {
            repository.mark_left(incomplete.id.get()).await?;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod lookup_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn guild(id: &str, name: &str, permissions: &str) -> UserGuild {
        UserGuild {
            id: id.into(),
            name: name.into(),
            icon: None,
            permissions: permissions.into(),
            owner: false,
        }
    }

    #[test]
    fn installed_first_but_membership_and_permissions_still_required() {
        let user = vec![
            guild("1", "Alpha", "32"),
            guild("2", "Old name", "8"),
            guild("3", "No access", "0"),
        ];
        let bot = vec![
            BotGuild {
                guild_id: 2,
                name: Some("Zebra".into()),
                icon: Some("new-icon".into()),
            },
            BotGuild {
                guild_id: 3,
                name: None,
                icon: None,
            },
            BotGuild {
                guild_id: 4,
                name: Some("Not a member".into()),
                icon: None,
            },
        ];
        let result = managed_guilds(user, bot);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "2");
        assert_eq!(result[0].name, "Zebra");
        assert!(result[0].bot_installed);
        assert_eq!(result[1].id, "1");
        assert!(!result[1].bot_installed);
    }

    #[test]
    fn handles_owner_and_invalid_permission_bits() {
        let mut owner = guild("1", "Owner", "0");
        owner.owner = true;
        assert!(owner.can_manage());
        assert!(!guild("2", "Invalid", "oops").can_manage());
    }
}

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use database::repositories::admin::AdminRepository;
use database::settings::{
    CommandPrefix, CustomCommandName, CustomCommandText, TranslationLanguage,
};
use database::{
    CustomCommandRepository, GuildDirectoryRepository, GuildSettingsRepository,
    ModerationRepository, NewPunishmentLadderRule,
};
use feature_guilds::{
    Freshness, GuildError, ManagedGuild, fresh_user_guild, managed_guilds, user_guilds,
};
use feature_moderation::{LadderRuleConfig, validate_ladder_rule_config};
use feature_settings::{DEFAULT_PREFIX, social_platform_keys, validate_social_platform};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::AuthSession;

#[derive(Debug)]
pub(crate) enum ApiError {
    Forbidden(&'static str),
    BadRequest(String),
    NotFound(&'static str),
    Upstream(&'static str),
    Database(database::DatabaseError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, message.to_owned()),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message.to_owned()),
            Self::Upstream(message) => (StatusCode::BAD_GATEWAY, message.to_owned()),
            Self::Database(error) => {
                tracing::error!(error = %error, "database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database operation failed".to_owned(),
                )
            }
        };

        (status, message).into_response()
    }
}

impl From<database::DatabaseError> for ApiError {
    fn from(error: database::DatabaseError) -> Self {
        Self::Database(error)
    }
}

impl From<GuildError> for ApiError {
    fn from(error: GuildError) -> Self {
        match error {
            GuildError::Unauthorized => {
                Self::Forbidden("Discord access is missing or expired. Sign in again.")
            }
            GuildError::Database(error) => Self::Database(error),
            _ => {
                tracing::warn!(error = %error, "guild lookup failed");
                Self::Upstream("Unable to load Discord guilds. Try again shortly.")
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuildSettingsResponse {
    guild: ManagedGuild,
    command_prefix: Option<String>,
    active_prefix: String,
    translation_language: Option<String>,
    active_translation_language: String,
    social_embeds: Vec<SocialEmbedSetting>,
    custom_commands: Vec<CustomCommandResponse>,
    ladder_rules: Vec<LadderRuleResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialEmbedSetting {
    platform: String,
    enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomCommandResponse {
    name: String,
    response: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LadderRuleResponse {
    id: u64,
    warning_threshold: u32,
    window_seconds: u64,
    action: String,
    duration_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGeneralSettingsRequest {
    command_prefix: Option<String>,
    translation_language: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSocialEmbedsRequest {
    disabled_platforms: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCustomCommandRequest {
    name: String,
    response: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertLadderRuleRequest {
    warning_threshold: u32,
    window_seconds: u64,
    action: String,
    duration_seconds: Option<u64>,
}

pub async fn list_managed_guilds(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
) -> Result<Json<Vec<ManagedGuild>>, ApiError> {
    Ok(Json(
        load_managed_guilds(&state, &session, Freshness::Cached).await?,
    ))
}

pub async fn get_guild_settings(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Path(guild_id): Path<u64>,
) -> Result<Json<GuildSettingsResponse>, ApiError> {
    let guild = match admin_guild(&state, &session, guild_id).await? {
        Some(guild) => guild,
        None => find_installed_guild(
            load_managed_guilds(&state, &session, Freshness::Cached).await?,
            guild_id,
        )?,
    };
    Ok(Json(load_guild_settings(&state, guild_id, guild).await?))
}

pub async fn update_general_settings(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Path(guild_id): Path<u64>,
    Json(request): Json<UpdateGeneralSettingsRequest>,
) -> Result<Json<GuildSettingsResponse>, ApiError> {
    let guild = require_manage_guild(&state, &session, guild_id).await?;
    let repository = GuildSettingsRepository::new(&state.database);

    match validate_optional_prefix(request.command_prefix.as_deref())? {
        Some(prefix) => repository.upsert_prefix(guild_id, &prefix).await?,
        None => repository.clear_prefix(guild_id).await?,
    }

    match validate_optional_translation_language(request.translation_language.as_deref())? {
        Some(language) => {
            repository
                .upsert_translation_language(guild_id, &language)
                .await?
        }
        None => repository.clear_translation_language(guild_id).await?,
    }

    Ok(Json(load_guild_settings(&state, guild_id, guild).await?))
}

pub async fn update_social_embeds(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Path(guild_id): Path<u64>,
    Json(request): Json<UpdateSocialEmbedsRequest>,
) -> Result<Json<GuildSettingsResponse>, ApiError> {
    let guild = require_manage_guild(&state, &session, guild_id).await?;
    let disabled = request
        .disabled_platforms
        .into_iter()
        .map(|platform| platform.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();

    if disabled
        .iter()
        .any(|platform| validate_social_platform(platform).is_none())
    {
        return Err(ApiError::BadRequest("Unknown social platform.".to_owned()));
    }

    let repository = GuildSettingsRepository::new(&state.database);
    for platform in social_platform_keys() {
        repository
            .set_social_embed_enabled(
                guild_id,
                platform,
                !disabled.iter().any(|item| item == platform),
            )
            .await?;
    }

    Ok(Json(load_guild_settings(&state, guild_id, guild).await?))
}

pub async fn upsert_custom_command(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Path(guild_id): Path<u64>,
    Json(request): Json<UpsertCustomCommandRequest>,
) -> Result<Json<GuildSettingsResponse>, ApiError> {
    let guild = require_manage_guild(&state, &session, guild_id).await?;
    let name = validate_command_name(&request.name)?;
    let response = CustomCommandText::parse(&request.response).ok_or_else(|| {
        ApiError::BadRequest("Command response must be 1-2,000 characters.".to_owned())
    })?;

    CustomCommandRepository::new(&state.database)
        .upsert(guild_id, &name, &response)
        .await?;

    Ok(Json(load_guild_settings(&state, guild_id, guild).await?))
}

pub async fn delete_custom_command(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Path((guild_id, name)): Path<(u64, String)>,
) -> Result<Json<GuildSettingsResponse>, ApiError> {
    let guild = require_manage_guild(&state, &session, guild_id).await?;
    let name = validate_command_name(&name)?;
    if !CustomCommandRepository::new(&state.database)
        .remove(guild_id, name.as_str())
        .await?
    {
        return Err(ApiError::NotFound("custom command not found"));
    }

    Ok(Json(load_guild_settings(&state, guild_id, guild).await?))
}

pub async fn create_ladder_rule(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Path(guild_id): Path<u64>,
    Json(request): Json<UpsertLadderRuleRequest>,
) -> Result<Json<GuildSettingsResponse>, ApiError> {
    let guild = require_manage_guild(&state, &session, guild_id).await?;
    let rule = validate_ladder_rule(request)?;
    ModerationRepository::new(&state.database)
        .create_ladder_rule(NewPunishmentLadderRule {
            guild_id,
            warning_threshold: rule.warning_threshold,
            window_seconds: rule.window_seconds,
            action: &rule.action,
            duration_seconds: rule.duration_seconds,
        })
        .await?;

    Ok(Json(load_guild_settings(&state, guild_id, guild).await?))
}

pub async fn update_ladder_rule(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Path((guild_id, rule_id)): Path<(u64, u64)>,
    Json(request): Json<UpsertLadderRuleRequest>,
) -> Result<Json<GuildSettingsResponse>, ApiError> {
    let guild = require_manage_guild(&state, &session, guild_id).await?;
    let rule = validate_ladder_rule(request)?;
    ModerationRepository::new(&state.database)
        .update_ladder_rule(
            guild_id,
            rule_id,
            rule.warning_threshold,
            rule.window_seconds,
            &rule.action,
            rule.duration_seconds,
        )
        .await?
        .ok_or(ApiError::NotFound("ladder rule not found"))?;

    Ok(Json(load_guild_settings(&state, guild_id, guild).await?))
}

pub async fn delete_ladder_rule(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Path((guild_id, rule_id)): Path<(u64, u64)>,
) -> Result<Json<GuildSettingsResponse>, ApiError> {
    let guild = require_manage_guild(&state, &session, guild_id).await?;
    if !ModerationRepository::new(&state.database)
        .delete_ladder_rule(guild_id, rule_id)
        .await?
    {
        return Err(ApiError::NotFound("ladder rule not found"));
    }

    Ok(Json(load_guild_settings(&state, guild_id, guild).await?))
}

async fn load_guild_settings(
    state: &AppState,
    guild_id: u64,
    guild: ManagedGuild,
) -> Result<GuildSettingsResponse, ApiError> {
    let settings_repository = GuildSettingsRepository::new(&state.database);
    let settings = settings_repository.find_by_guild_id(guild_id).await?;
    let disabled_social_platforms = settings_repository
        .disabled_social_platforms(guild_id)
        .await?;
    let custom_commands = CustomCommandRepository::new(&state.database)
        .list_entries(guild_id)
        .await?
        .into_iter()
        .map(|command| CustomCommandResponse {
            name: command.name,
            response: command.response,
        })
        .collect();
    let ladder_rules = ModerationRepository::new(&state.database)
        .ladder_rules(guild_id)
        .await?
        .into_iter()
        .map(|rule| LadderRuleResponse {
            id: rule.id,
            warning_threshold: rule.warning_threshold,
            window_seconds: rule.window_seconds,
            action: rule.action,
            duration_seconds: rule.duration_seconds,
        })
        .collect();
    let command_prefix = settings
        .as_ref()
        .and_then(|settings| settings.command_prefix.clone());
    let translation_language = settings
        .as_ref()
        .and_then(|settings| settings.translation_language.clone());

    Ok(GuildSettingsResponse {
        guild,
        active_prefix: command_prefix
            .as_deref()
            .unwrap_or(DEFAULT_PREFIX)
            .to_owned(),
        command_prefix,
        active_translation_language: translation_language.as_deref().unwrap_or("en").to_owned(),
        translation_language,
        social_embeds: social_platform_keys()
            .map(|platform| SocialEmbedSetting {
                platform: platform.to_owned(),
                enabled: !disabled_social_platforms
                    .iter()
                    .any(|disabled| disabled == platform),
            })
            .collect(),
        custom_commands,
        ladder_rules,
    })
}

async fn load_managed_guilds(
    state: &AppState,
    session: &AuthSession,
    freshness: Freshness,
) -> Result<Vec<ManagedGuild>, ApiError> {
    let guilds = user_guilds(&state.database, &state.http, &session.user.id, freshness).await?;
    let installed = GuildDirectoryRepository::new(&state.database)
        .installed()
        .await?;
    Ok(managed_guilds(guilds, installed))
}

fn find_installed_guild(
    guilds: Vec<ManagedGuild>,
    guild_id: u64,
) -> Result<ManagedGuild, ApiError> {
    let guild = guilds
        .into_iter()
        .find(|guild| guild.id == guild_id.to_string())
        .ok_or(ApiError::Forbidden(
            "You need Manage Server or Administrator permission.",
        ))?;
    if !guild.bot_installed {
        return Err(ApiError::Forbidden(
            "Add Yin to this server before editing settings.",
        ));
    }
    Ok(guild)
}

// Mutations always check Discord again. A cached read must never authorize a write.
async fn require_manage_guild(
    state: &AppState,
    session: &AuthSession,
    guild_id: u64,
) -> Result<ManagedGuild, ApiError> {
    if let Some(guild) = admin_guild(state, session, guild_id).await? {
        return Ok(guild);
    }
    let installed = GuildDirectoryRepository::new(&state.database)
        .installed_guild(guild_id)
        .await?
        .ok_or(ApiError::Forbidden(
            "Add Yin to this server before editing settings.",
        ))?;
    let guild = fresh_user_guild(&state.database, &state.http, &session.user.id, guild_id)
        .await?
        .ok_or(ApiError::Forbidden(
            "You need Manage Server or Administrator permission.",
        ))?;
    find_installed_guild(managed_guilds(vec![guild], vec![installed]), guild_id)
}

// Bot admins can edit any guild the bot is in, without Discord permissions there.
async fn admin_guild(
    state: &AppState,
    session: &AuthSession,
    guild_id: u64,
) -> Result<Option<ManagedGuild>, ApiError> {
    if !AdminRepository::new(&state.database)
        .is_admin(&session.user.id)
        .await?
    {
        return Ok(None);
    }
    let guild = GuildDirectoryRepository::new(&state.database)
        .installed_guild(guild_id)
        .await?
        .ok_or(ApiError::Forbidden(
            "Add Yin to this server before editing settings.",
        ))?;
    Ok(Some(ManagedGuild {
        id: guild_id.to_string(),
        name: guild.name.unwrap_or_else(|| guild_id.to_string()),
        icon: guild.icon,
        bot_installed: true,
    }))
}

fn validate_optional_prefix(prefix: Option<&str>) -> Result<Option<CommandPrefix>, ApiError> {
    prefix
        .filter(|prefix| !prefix.trim().is_empty())
        .map(|prefix| {
            CommandPrefix::parse(prefix).ok_or_else(|| {
                ApiError::BadRequest(
                    "Prefixes must be at most 16 characters and contain no whitespace.".to_owned(),
                )
            })
        })
        .transpose()
}

fn validate_optional_translation_language(
    language: Option<&str>,
) -> Result<Option<TranslationLanguage>, ApiError> {
    language
        .filter(|language| !language.trim().is_empty())
        .map(|language| {
            TranslationLanguage::parse(language).ok_or_else(|| {
                ApiError::BadRequest(
                    "Use a valid ISO language code such as en, ja, pt-BR, or zh-Hant.".to_owned(),
                )
            })
        })
        .transpose()
}

fn validate_command_name(name: &str) -> Result<CustomCommandName, ApiError> {
    CustomCommandName::parse(name).ok_or_else(|| {
        ApiError::BadRequest(
            "Names must be 1-32 characters using only letters, numbers, `_`, or `-`.".to_owned(),
        )
    })
}

fn validate_ladder_rule(request: UpsertLadderRuleRequest) -> Result<LadderRuleConfig, ApiError> {
    validate_ladder_rule_config(
        request.warning_threshold,
        request.window_seconds,
        &request.action,
        request.duration_seconds,
    )
    .map_err(ApiError::BadRequest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_settings_preserve_reset_semantics() {
        for input in [None, Some(""), Some(" \t\n ")] {
            assert_eq!(validate_optional_prefix(input).unwrap(), None);
            assert_eq!(validate_optional_translation_language(input).unwrap(), None);
        }
    }

    #[test]
    fn optional_settings_return_normalized_values() {
        let prefix: CommandPrefix = validate_optional_prefix(Some(" ! ")).unwrap().unwrap();
        assert_eq!(prefix.as_str(), "!");
        let language: TranslationLanguage = validate_optional_translation_language(Some(" PT_BR "))
            .unwrap()
            .unwrap();
        assert_eq!(language.as_str(), "pt-br");
    }

    #[test]
    fn invalid_settings_are_bad_requests() {
        assert!(matches!(
            validate_optional_prefix(Some("two words")),
            Err(ApiError::BadRequest(_))
        ));
        assert!(matches!(
            validate_optional_translation_language(Some("unknown")),
            Err(ApiError::BadRequest(_))
        ));
        assert!(matches!(
            validate_command_name("two words"),
            Err(ApiError::BadRequest(_))
        ));
    }
}

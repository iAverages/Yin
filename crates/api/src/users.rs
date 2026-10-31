use axum::Json;
use axum::extract::State;
use database::{GuildDirectoryRepository, UserSocialEmbedsRepository};
use feature_settings::{SocialDisplay, social_platform_keys, validate_social_platform};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::AuthSession;
use crate::guilds::ApiError;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsResponse {
    social_embeds: Vec<UserSocialEmbedSetting>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSocialEmbedSetting {
    platform: String,
    enabled: bool,
    display: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserSocialEmbedsRequest {
    social_embeds: Vec<UserSocialEmbedSetting>,
}

pub async fn get_user_settings(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let user_id = discord_user_id(&state, &session).await?;
    Ok(Json(load_user_settings(&state, user_id).await?))
}

pub async fn update_user_social_embeds(
    State(state): State<AppState>,
    axum::Extension(session): axum::Extension<AuthSession>,
    Json(request): Json<UpdateUserSocialEmbedsRequest>,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let user_id = discord_user_id(&state, &session).await?;
    let settings = request
        .social_embeds
        .iter()
        .map(|setting| {
            let platform = validate_social_platform(&setting.platform)
                .ok_or_else(|| ApiError::BadRequest("Unknown social platform.".to_owned()))?;
            let display = SocialDisplay::from_key(&setting.display)
                .ok_or_else(|| ApiError::BadRequest("Unknown embed display.".to_owned()))?;
            Ok((platform, setting.enabled, display))
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    let repository = UserSocialEmbedsRepository::new(&state.database);
    for (platform, enabled, display) in settings {
        repository.set_enabled(user_id, platform, enabled).await?;
        repository
            .set_display(
                user_id,
                platform,
                (display != SocialDisplay::Full).then(|| display.key()),
            )
            .await?;
    }

    Ok(Json(load_user_settings(&state, user_id).await?))
}

async fn load_user_settings(
    state: &AppState,
    user_id: u64,
) -> Result<UserSettingsResponse, ApiError> {
    let repository = UserSocialEmbedsRepository::new(&state.database);
    let disabled = repository.disabled_platforms(user_id).await?;
    let displays = repository.displays(user_id).await?;
    Ok(UserSettingsResponse {
        social_embeds: social_platform_keys()
            .map(|platform| UserSocialEmbedSetting {
                platform: platform.to_owned(),
                enabled: !disabled.iter().any(|disabled| disabled == platform),
                display: displays
                    .iter()
                    .find(|(display_platform, _)| display_platform == platform)
                    .map_or(SocialDisplay::Full.key(), |(_, display)| display)
                    .to_owned(),
            })
            .collect(),
    })
}

// Bot settings are keyed by Discord user ID, not the better-auth user ID.
async fn discord_user_id(state: &AppState, session: &AuthSession) -> Result<u64, ApiError> {
    GuildDirectoryRepository::new(&state.database)
        .discord_user_id(&session.user.id)
        .await?
        .ok_or(ApiError::Forbidden(
            "Sign in with Discord to manage your settings.",
        ))
}

#[cfg(test)]
mod tests;

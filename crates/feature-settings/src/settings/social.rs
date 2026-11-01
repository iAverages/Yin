use bot_core::command::CommandContext;
use bot_core::response::{Embed, EmbedKind};
use bot_core::{BotState, Error};
use database::{Database, GuildSettingsRepository, UserSocialEmbedsRepository};
use feature_social::{SocialDisplay, SocialPlatform};
use serde_json::{Value, json};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;

pub(super) const PAGE_ID: &str = "settings:social";
const CUSTOM_ID_PREFIX: &str = "settings:social:";
const MODES: [(&str, &str); 3] = [
    ("off", "Off"),
    ("full", "Full"),
    ("media-only", "Media only"),
];

pub fn social_platform_keys() -> impl Iterator<Item = &'static str> {
    SocialPlatform::ALL.into_iter().map(SocialPlatform::key)
}

pub fn validate_social_platform(platform: &str) -> Option<&'static str> {
    let platform = platform.trim().to_ascii_lowercase();
    social_platform_keys().find(|key| *key == platform)
}

pub(super) async fn user_components(
    database: &Database,
    user_id: u64,
) -> Result<Vec<Value>, Error> {
    let repository = UserSocialEmbedsRepository::new(database);
    Ok(social_components(
        &repository.disabled_platforms(user_id).await?,
        &repository.displays(user_id).await?,
    ))
}

fn social_components(disabled: &[String], displays: &[(String, String)]) -> Vec<Value> {
    let mut components = vec![json!({
        "type": 10,
        "content": "## Social embeds\nHow Yin embeds your links in every server. \
            Media only sends just the images and videos.",
    })];
    for platform in SocialPlatform::ALL {
        let key = platform.key();
        let current = if disabled.iter().any(|disabled| disabled == key) {
            "off"
        } else {
            displays
                .iter()
                .find(|(display_platform, _)| display_platform == key)
                .map_or(SocialDisplay::Full.key(), |(_, display)| display)
        };
        components.push(json!({"type": 10, "content": format!("**{}**", label(platform))}));
        components.push(json!({
            "type": 1,
            "components": [{
                "type": 3,
                "custom_id": format!("{CUSTOM_ID_PREFIX}{key}"),
                "options": MODES.map(|(value, label)| json!({
                    "label": label,
                    "value": value,
                    "default": value == current,
                })),
            }],
        }));
    }
    components.push(json!({
        "type": 1,
        "components": [{"type": 2, "style": 2, "label": "Back", "custom_id": super::HOME_ID}],
    }));
    vec![json!({"type": 17, "accent_color": super::ACCENT_COLOR, "components": components})]
}

fn label(platform: SocialPlatform) -> &'static str {
    match platform {
        SocialPlatform::Twitter => "Twitter / X",
        SocialPlatform::Bluesky => "Bluesky",
        SocialPlatform::Instagram => "Instagram",
        SocialPlatform::Facebook => "Facebook",
        SocialPlatform::TikTok => "TikTok",
        SocialPlatform::Reddit => "Reddit",
        SocialPlatform::Spotify => "Spotify",
    }
}

/// Returns the page to show, or `None` if the interaction is not for this page.
pub(super) async fn handle_interaction(
    data: &BotState,
    interaction: &Interaction,
    component: &MessageComponentInteractionData,
) -> Result<Option<Vec<Value>>, Error> {
    let Some(user_id) = interaction.author_id().map(|id| id.get()) else {
        return Ok(None);
    };
    if component.custom_id == PAGE_ID {
        return Ok(Some(user_components(&data.database, user_id).await?));
    }
    let Some(platform) = component
        .custom_id
        .strip_prefix(CUSTOM_ID_PREFIX)
        .and_then(validate_social_platform)
    else {
        return Ok(None);
    };
    let repository = UserSocialEmbedsRepository::new(&data.database);
    match component.values.first().map(String::as_str) {
        // Off keeps the saved display so turning a platform back on restores it.
        Some("off") => repository.set_enabled(user_id, platform, false).await?,
        Some(display) => {
            let Some(display) = SocialDisplay::from_key(display) else {
                return Ok(None);
            };
            repository.set_enabled(user_id, platform, true).await?;
            repository
                .set_display(
                    user_id,
                    platform,
                    (display != SocialDisplay::Full).then(|| display.key()),
                )
                .await?;
        }
        None => return Ok(None),
    }
    Ok(Some(user_components(&data.database, user_id).await?))
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "social", desc = "Manage server-wide social embed settings.")]
pub enum ServerSocialCommand {
    #[command(name = "view")]
    View(ServerView),
    #[command(name = "set")]
    Set(ServerSet),
    #[command(name = "reset")]
    Reset(ServerReset),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "view", desc = "View this server's social embed settings.")]
pub struct ServerView;

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "set",
    desc = "Set this server's social embed preference for a platform."
)]
pub struct ServerSet {
    /// Social platform to configure
    platform: SocialPlatform,
    /// Whether the bot should embed links from this platform
    enabled: bool,
}

#[derive(CommandModel, CreateCommand)]
#[command(
    name = "reset",
    desc = "Reset this server's social embed preferences to enabled by default."
)]
pub struct ServerReset {
    /// Platform to reset; omit to reset all platforms
    platform: Option<SocialPlatform>,
}

pub async fn run(ctx: &CommandContext, command: ServerSocialCommand) -> Result<(), Error> {
    ctx.set_ephemeral();
    let guild_id = ctx
        .guild_id()
        .expect("guild-only command has a guild ID")
        .get();
    let repository = GuildSettingsRepository::new(&ctx.state.database);

    match command {
        ServerSocialCommand::View(_) => {
            let disabled = repository.disabled_social_platforms(guild_id).await?;
            let mut embed = Embed::new(EmbedKind::Info, "Server Social Embeds").description(
                "Applies to everyone's links in this server. Personal opt-outs still apply.",
            );
            for platform in SocialPlatform::ALL {
                let enabled = !disabled.iter().any(|disabled| disabled == platform.key());
                embed = embed.field(
                    platform.key(),
                    if enabled { "Enabled" } else { "Disabled" },
                    true,
                );
            }
            ctx.send(embed).await
        }
        ServerSocialCommand::Set(ServerSet { platform, enabled }) => {
            repository
                .set_social_embed_enabled(guild_id, platform.key(), enabled)
                .await?;
            ctx.send(
                Embed::new(EmbedKind::Success, "Server Social Embeds Updated")
                    .description(
                        "Applies to everyone's links in this server. Personal opt-outs still apply.",
                    )
                    .field(
                        platform.key(),
                        if enabled { "Enabled" } else { "Disabled" },
                        true,
                    ),
            )
            .await
        }
        ServerSocialCommand::Reset(ServerReset { platform }) => {
            let platform = platform.map(SocialPlatform::key);
            repository.reset_social_embeds(guild_id, platform).await?;
            ctx.send(
                Embed::new(EmbedKind::Success, "Server Social Embeds Reset")
                    .description("Personal opt-outs still apply to links in this server.")
                    .field(
                        platform.unwrap_or("All platforms"),
                        "Enabled (default)",
                        true,
                    ),
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn social_components_select_the_saved_mode_per_platform() {
        let components = social_components(
            &["tiktok".to_owned()],
            &[("twitter".to_owned(), "media-only".to_owned())],
        );
        let selected = |key: &str| {
            components[0]["components"]
                .as_array()
                .unwrap()
                .iter()
                .find_map(|child| {
                    let select = &child["components"][0];
                    (select["custom_id"] == format!("{CUSTOM_ID_PREFIX}{key}")).then(|| {
                        select["options"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|option| option["default"] == true)
                            .unwrap()["value"]
                            .clone()
                    })
                })
                .unwrap()
        };

        assert_eq!(selected("twitter"), "media-only");
        assert_eq!(selected("tiktok"), "off");
        assert_eq!(selected("spotify"), "full");
    }
}

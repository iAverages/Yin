use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};
use database::{GuildSettingsRepository, UserSocialEmbedsRepository};
use feature_social::SocialPlatform;

#[derive(Clone, Copy, Default, poise::ChoiceParameter)]
pub enum SocialScope {
    #[default]
    #[name = "user"]
    User,
    #[name = "server"]
    Server,
}

pub fn social_platform_keys() -> impl Iterator<Item = &'static str> {
    SocialPlatform::ALL.into_iter().map(SocialPlatform::key)
}

pub fn validate_social_platform(platform: &str) -> Option<&'static str> {
    let platform = platform.trim().to_ascii_lowercase();
    social_platform_keys().find(|key| *key == platform)
}

/// Manage personal or server-wide social embed preferences.
#[poise::command(
    prefix_command,
    slash_command,
    ephemeral,
    subcommands("view", "set"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn social(ctx: Context<'_>) -> Result<(), Error> {
    response::send(
        ctx,
        Embed::new(EmbedKind::Info, "Social Embeds").description(
            "Use `/settings social view` to see your preferences or `/settings social set` to change them. \
             Add `scope:server` to manage server-wide settings (changes require Manage Server). \
             Prefix commands: `!settings social view [user|server]` and \
             `!settings social set <platform> <true|false> [user|server]`.",
        ),
    )
    .await
}

/// View personal or server-wide social embed preferences.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    ephemeral,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn view(
    ctx: Context<'_>,
    #[description = "View your preferences (default) or this server's settings"] scope: Option<
        SocialScope,
    >,
) -> Result<(), Error> {
    let (disabled, title, description) = match scope.unwrap_or_default() {
        SocialScope::User => (
            UserSocialEmbedsRepository::new(&ctx.data().database)
                .disabled_platforms(ctx.author().id.get())
                .await?,
            "Your Social Embeds",
            "Applies to your links across all servers. Server settings may also disable embeds.",
        ),
        SocialScope::Server => (
            GuildSettingsRepository::new(&ctx.data().database)
                .disabled_social_platforms(
                    ctx.guild_id()
                        .expect("guild-only command has a guild ID")
                        .get(),
                )
                .await?,
            "Server Social Embeds",
            "Applies to everyone's links in this server. Personal opt-outs still apply.",
        ),
    };
    let mut embed = Embed::new(EmbedKind::Info, title).description(description);
    for platform in SocialPlatform::ALL {
        let enabled = !disabled.iter().any(|disabled| disabled == platform.key());
        embed = embed.field(
            platform.key(),
            if enabled { "Enabled" } else { "Disabled" },
            true,
        );
    }
    response::send(ctx, embed).await
}

/// Enable or disable personal or server-wide embeds for a social platform.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    ephemeral,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Social platform to configure"] platform: SocialPlatform,
    #[description = "Whether the bot should embed links from this platform"] enabled: bool,
    #[description = "Change your preferences (default) or this server's settings"] scope: Option<
        SocialScope,
    >,
) -> Result<(), Error> {
    let (title, description) = match scope.unwrap_or_default() {
        SocialScope::User => {
            UserSocialEmbedsRepository::new(&ctx.data().database)
                .set_enabled(ctx.author().id.get(), platform.key(), enabled)
                .await?;
            (
                "Your Social Embeds Updated",
                "Applies to your links across all servers. Server settings may also disable embeds.",
            )
        }
        SocialScope::Server => {
            if !bot_core::permissions::require_manage_guild(ctx).await? {
                return response::error(
                    ctx,
                    "You need Manage Server permission to change server social embeds.",
                )
                .await;
            }
            GuildSettingsRepository::new(&ctx.data().database)
                .set_social_embed_enabled(
                    ctx.guild_id()
                        .expect("guild-only command has a guild ID")
                        .get(),
                    platform.key(),
                    enabled,
                )
                .await?;
            (
                "Server Social Embeds Updated",
                "Applies to everyone's links in this server. Personal opt-outs still apply.",
            )
        }
    };
    response::send(
        ctx,
        Embed::new(EmbedKind::Success, title)
            .description(description)
            .field(
                platform.key(),
                if enabled { "Enabled" } else { "Disabled" },
                true,
            ),
    )
    .await
}

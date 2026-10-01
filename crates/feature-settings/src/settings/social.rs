use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};
use database::{GuildSettingsRepository, UserSocialEmbedsRepository};
use feature_social::SocialPlatform;

enum SocialScope {
    User,
    Server,
}

pub fn social_platform_keys() -> impl Iterator<Item = &'static str> {
    SocialPlatform::ALL.into_iter().map(SocialPlatform::key)
}

pub fn validate_social_platform(platform: &str) -> Option<&'static str> {
    let platform = platform.trim().to_ascii_lowercase();
    social_platform_keys().find(|key| *key == platform)
}

/// Manage your social embed preferences across all servers.
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    ephemeral,
    subcommand_required,
    subcommands("view", "set", "reset"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn social(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View your social embed preferences.
#[poise::command(prefix_command, slash_command, guild_only, ephemeral)]
async fn view(ctx: Context<'_>) -> Result<(), Error> {
    view_settings(ctx, SocialScope::User).await
}

/// Set your social embed preference for a platform.
#[poise::command(prefix_command, slash_command, guild_only, ephemeral)]
async fn set(
    ctx: Context<'_>,
    #[description = "Social platform to configure"] platform: SocialPlatform,
    #[description = "Whether the bot should embed your links from this platform"] enabled: bool,
) -> Result<(), Error> {
    set_settings(ctx, platform, enabled, SocialScope::User).await
}

/// Reset your social embed preferences to enabled by default.
#[poise::command(prefix_command, slash_command, guild_only, ephemeral)]
async fn reset(
    ctx: Context<'_>,
    #[description = "Platform to reset; omit to reset all platforms"] platform: Option<
        SocialPlatform,
    >,
) -> Result<(), Error> {
    reset_settings(ctx, platform, SocialScope::User).await
}

async fn view_settings(ctx: Context<'_>, scope: SocialScope) -> Result<(), Error> {
    let (disabled, title, description) = match scope {
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

async fn set_settings(
    ctx: Context<'_>,
    platform: SocialPlatform,
    enabled: bool,
    scope: SocialScope,
) -> Result<(), Error> {
    let (title, description) = match scope {
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

async fn reset_settings(
    ctx: Context<'_>,
    platform: Option<SocialPlatform>,
    scope: SocialScope,
) -> Result<(), Error> {
    let platform = platform.map(SocialPlatform::key);
    let (title, description) = match scope {
        SocialScope::User => {
            UserSocialEmbedsRepository::new(&ctx.data().database)
                .reset(ctx.author().id.get(), platform)
                .await?;
            (
                "Your Social Embeds Reset",
                "Server settings may still disable embeds for your links.",
            )
        }
        SocialScope::Server => {
            GuildSettingsRepository::new(&ctx.data().database)
                .reset_social_embeds(
                    ctx.guild_id()
                        .expect("guild-only command has a guild ID")
                        .get(),
                    platform,
                )
                .await?;
            (
                "Server Social Embeds Reset",
                "Personal opt-outs still apply to links in this server.",
            )
        }
    };
    response::send(
        ctx,
        Embed::new(EmbedKind::Success, title)
            .description(description)
            .field(
                platform.unwrap_or("All platforms"),
                "Enabled (default)",
                true,
            ),
    )
    .await
}

/// Manage server-wide social embed settings.
#[poise::command(
    prefix_command,
    slash_command,
    rename = "social",
    guild_only,
    ephemeral,
    subcommand_required,
    subcommands("server_view", "server_set", "server_reset"),
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn server_social(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View this server's social embed settings.
#[poise::command(prefix_command, slash_command, rename = "view", guild_only, ephemeral)]
async fn server_view(ctx: Context<'_>) -> Result<(), Error> {
    view_settings(ctx, SocialScope::Server).await
}

/// Set this server's social embed preference for a platform.
#[poise::command(prefix_command, slash_command, rename = "set", guild_only, ephemeral)]
async fn server_set(
    ctx: Context<'_>,
    #[description = "Social platform to configure"] platform: SocialPlatform,
    #[description = "Whether the bot should embed links from this platform"] enabled: bool,
) -> Result<(), Error> {
    set_settings(ctx, platform, enabled, SocialScope::Server).await
}

/// Reset this server's social embed preferences to enabled by default.
#[poise::command(prefix_command, slash_command, rename = "reset", guild_only, ephemeral)]
async fn server_reset(
    ctx: Context<'_>,
    #[description = "Platform to reset; omit to reset all platforms"] platform: Option<
        SocialPlatform,
    >,
) -> Result<(), Error> {
    reset_settings(ctx, platform, SocialScope::Server).await
}

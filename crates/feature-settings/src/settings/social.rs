use bot_core::response::{self, Embed, EmbedKind};
use bot_core::{Context, Error, poise};
use database::UserSocialEmbedsRepository;
use feature_social::SocialPlatform;

/// Manage your social embed preferences across all servers.
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
        Embed::new(EmbedKind::Info, "Your Social Embeds").description(
            "Use `/settings social view` to see your preferences or `/settings social set` to change them. \
             Prefix commands: `!settings social view` and `!settings social set <platform> <true|false>`.",
        ),
    )
    .await
}

/// View your social embed preferences across all servers.
#[poise::command(
    prefix_command,
    slash_command,
    ephemeral,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn view(ctx: Context<'_>) -> Result<(), Error> {
    let disabled = UserSocialEmbedsRepository::new(&ctx.data().database)
        .disabled_platforms(ctx.author().id.get())
        .await?;
    let mut embed = Embed::new(EmbedKind::Info, "Your Social Embeds").description(
        "These preferences apply to your links across all servers. \
         Use `/settings social set` or `!settings social set <platform> <true|false>` to change them.",
    );
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

/// Enable or disable bot embeds for your links from a social platform.
#[poise::command(
    prefix_command,
    slash_command,
    ephemeral,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Social platform to configure"] platform: SocialPlatform,
    #[description = "Whether the bot should embed your links from this platform"] enabled: bool,
) -> Result<(), Error> {
    UserSocialEmbedsRepository::new(&ctx.data().database)
        .set_enabled(ctx.author().id.get(), platform.key(), enabled)
        .await?;
    response::send(
        ctx,
        Embed::new(EmbedKind::Success, "Your Social Embeds Updated")
            .description("Applies to your links across all servers.")
            .field(
                platform.key(),
                if enabled { "Enabled" } else { "Disabled" },
                true,
            ),
    )
    .await
}

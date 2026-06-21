use bot_core::response::{self, Embed, EmbedKind};
use bot_core::time;
use bot_core::{Context, Error, poise, serenity};

#[poise::command(
    slash_command,
    prefix_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "User to inspect"] user: Option<serenity::User>,
    #[description = "Raw Discord user ID"] id: Option<String>,
) -> Result<(), Error> {
    if user.is_some() && id.is_some() {
        return response::error(ctx, "Provide either a user or an ID, not both.").await;
    }

    let target = if let Some(user) = user {
        user
    } else if let Some(id) = id {
        let Some(user) = fetch_user_by_id(ctx, &id).await? else {
            return Ok(());
        };
        user
    } else {
        ctx.author().clone()
    };

    send_user_info(ctx, target).await
}

#[poise::command(
    context_menu_command = "User Info",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn user_context(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    send_user_info(ctx, user).await
}

async fn fetch_user_by_id(ctx: Context<'_>, id: &str) -> Result<Option<serenity::User>, Error> {
    let Ok(id) = id.trim().parse::<u64>() else {
        response::error(ctx, "User ID must be a valid Discord snowflake.").await?;
        return Ok(None);
    };

    match serenity::UserId::new(id)
        .to_user(ctx.serenity_context())
        .await
    {
        Ok(user) => Ok(Some(user)),
        Err(error) => {
            tracing::warn!(user_id = id, error = %error, "failed to fetch user by id");
            response::error(ctx, "That user could not be found or is not accessible.").await?;
            Ok(None)
        }
    }
}

async fn send_user_info(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let member = if let Some(guild_id) = ctx.guild_id() {
        guild_id.member(ctx.serenity_context(), user.id).await.ok()
    } else {
        None
    };

    let display_name = member
        .as_ref()
        .and_then(|member| member.nick.clone())
        .or_else(|| user.global_name.clone())
        .unwrap_or_else(|| user.name.clone());

    let mut embed = Embed::new(EmbedKind::Info, "User Info")
        .field("Username", user.name.clone(), true)
        .field("Display Name", display_name, true)
        .field("User ID", user.id.to_string(), true)
        .field("Bot Account", if user.bot { "Yes" } else { "No" }, true)
        .field(
            "Created",
            time::discord_timestamp(user.id.created_at()),
            false,
        )
        .thumbnail(user.face());

    if let Some(member) = member {
        if let Some(joined_at) = member.joined_at {
            embed = embed.field("Joined Server", time::discord_timestamp(joined_at), false);
        }
        embed = embed.field("Role Count", member.roles.len().to_string(), true);
    }

    response::send(ctx, embed).await
}

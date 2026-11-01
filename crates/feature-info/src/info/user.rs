use bot_core::Error;
use bot_core::cdn;
use bot_core::command::{CommandContext, CommandFuture};
use bot_core::response::{Embed, EmbedKind};
use bot_core::time;
use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};
use twilight_model::application::command::{Command as CommandDefinition, CommandType};
use twilight_model::application::interaction::InteractionContextType;
use twilight_model::id::Id;
use twilight_model::oauth::ApplicationIntegrationType;
use twilight_model::user::User;
use twilight_util::builder::command::CommandBuilder;
use twilight_util::snowflake::Snowflake;

#[derive(CommandModel, CreateCommand)]
#[command(name = "user", desc = "View information about a user")]
pub struct UserCommand {
    /// User to inspect
    user: Option<ResolvedUser>,
    /// Raw Discord user ID
    id: Option<String>,
}

pub async fn run(ctx: &CommandContext, command: UserCommand) -> Result<(), Error> {
    if command.user.is_some() && command.id.is_some() {
        return ctx.error("Provide either a user or an ID, not both.").await;
    }

    let target = if let Some(user) = command.user {
        user.resolved
    } else if let Some(id) = command.id {
        let Some(user) = fetch_user_by_id(ctx, &id).await? else {
            return Ok(());
        };
        user
    } else {
        ctx.author().clone()
    };

    send_user_info(ctx, target).await
}

pub fn user_context_definition() -> CommandDefinition {
    CommandBuilder::new("User Info", "", CommandType::User)
        .contexts([
            InteractionContextType::Guild,
            InteractionContextType::BotDm,
            InteractionContextType::PrivateChannel,
        ])
        .integration_types([
            ApplicationIntegrationType::GuildInstall,
            ApplicationIntegrationType::UserInstall,
        ])
        .build()
}

pub fn run_user_context(ctx: &CommandContext) -> CommandFuture<'_> {
    Box::pin(async move {
        let user = ctx
            .data
            .target_id
            .zip(ctx.data.resolved.as_ref())
            .and_then(|(target_id, resolved)| resolved.users.get(&target_id.cast()))
            .cloned()
            .ok_or("user context menu missing target user")?;
        send_user_info(ctx, user).await
    })
}

async fn fetch_user_by_id(ctx: &CommandContext, id: &str) -> Result<Option<User>, Error> {
    let Some(id) = id.trim().parse::<u64>().ok().and_then(Id::new_checked) else {
        ctx.error("User ID must be a valid Discord snowflake.")
            .await?;
        return Ok(None);
    };

    let user = async { Ok::<_, Error>(ctx.state.http.user(id).await?.model().await?) }.await;
    match user {
        Ok(user) => Ok(Some(user)),
        Err(error) => {
            tracing::warn!(user_id = %id, error = %error, "failed to fetch user by id");
            ctx.error("That user could not be found or is not accessible.")
                .await?;
            Ok(None)
        }
    }
}

async fn send_user_info(ctx: &CommandContext, user: User) -> Result<(), Error> {
    let member = match ctx.guild_id() {
        Some(guild_id) => async {
            Ok::<_, Error>(
                ctx.state
                    .http
                    .guild_member(guild_id, user.id)
                    .await?
                    .model()
                    .await?,
            )
        }
        .await
        .ok(),
        None => None,
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
            time::discord_timestamp(user.id.timestamp() / 1000),
            false,
        )
        .thumbnail(cdn::face(&user));

    if let Some(member) = member {
        if let Some(joined_at) = member.joined_at {
            embed = embed.field(
                "Joined Server",
                time::discord_timestamp(joined_at.as_secs()),
                false,
            );
        }
        embed = embed.field("Role Count", member.roles.len().to_string(), true);
    }

    ctx.send(embed).await
}

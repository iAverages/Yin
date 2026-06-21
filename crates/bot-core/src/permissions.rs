use crate::{Context, Error, serenity};

pub async fn require_manage_guild(ctx: Context<'_>) -> Result<bool, Error> {
    author_has_guild_permission(ctx, serenity::Permissions::MANAGE_GUILD).await
}

pub async fn author_has_guild_permission(
    ctx: Context<'_>,
    required: serenity::Permissions,
) -> Result<bool, Error> {
    if required.is_empty() {
        return Ok(true);
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Ok(false);
    };

    let guild = guild_id.to_partial_guild(ctx.serenity_context()).await?;
    if guild.owner_id == ctx.author().id {
        return Ok(true);
    }

    let member = guild_id
        .member(ctx.serenity_context(), ctx.author().id)
        .await?;
    let mut permissions = guild
        .roles
        .get(&serenity::RoleId::new(guild_id.get()))
        .map(|role| role.permissions)
        .unwrap_or_else(serenity::Permissions::empty);

    for role_id in member.roles {
        if let Some(role) = guild.roles.get(&role_id) {
            permissions |= role.permissions;
        }
    }

    Ok(
        permissions.contains(serenity::Permissions::ADMINISTRATOR)
            || permissions.contains(required),
    )
}

use crate::{Context, Error, serenity};
use std::collections::HashMap;

pub async fn require_manage_guild(ctx: Context<'_>) -> Result<bool, Error> {
    author_has_guild_permission(ctx, serenity::Permissions::MANAGE_GUILD).await
}

pub async fn can_moderate_member(
    ctx: Context<'_>,
    target: &serenity::Member,
) -> Result<bool, Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(false);
    };
    if target.user.id == ctx.author().id {
        return Ok(false);
    }

    let guild = guild_id.to_partial_guild(ctx.serenity_context()).await?;
    if target.user.id == guild.owner_id {
        return Ok(false);
    }
    let actor = guild_id
        .member(ctx.serenity_context(), ctx.author().id)
        .await?;
    let bot_user_id = { ctx.serenity_context().cache.current_user().id };
    let bot = guild_id.member(ctx.serenity_context(), bot_user_id).await?;

    Ok(
        (ctx.author().id == guild.owner_id || member_is_above(&guild.roles, &actor, target))
            && member_is_above(&guild.roles, &bot, target),
    )
}

pub fn member_is_above(
    roles: &HashMap<serenity::RoleId, serenity::Role>,
    actor: &serenity::Member,
    target: &serenity::Member,
) -> bool {
    highest_role(roles, actor) > highest_role(roles, target)
}

fn highest_role(
    roles: &HashMap<serenity::RoleId, serenity::Role>,
    member: &serenity::Member,
) -> (u16, std::cmp::Reverse<u64>) {
    member
        .roles
        .iter()
        .filter_map(|id| roles.get(id))
        .map(|role| (role.position, std::cmp::Reverse(role.id.get())))
        .max()
        .unwrap_or_default()
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

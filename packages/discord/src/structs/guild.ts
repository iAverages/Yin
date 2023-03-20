import { z } from "zod";

// Do not change the import order of these, they are placed this way to make
// things get imported in the correct order, I should probably move some things around
// but that fuckeds up the structure
import { userSchema } from "./user";
import { roleSchema } from "./role";
import { emojiSchema } from "./emoji";
import { stickerSchema } from "./sticker";

export const guildFeatures = [
    "ANIMATED_BANNER",
    "ANIMATED_ICON",
    "APPLICATION_COMMAND_PERMISSIONS_V2",
    "AUTO_MODERATION",
    "BANNER",
    "COMMUNITY",
    "CREATOR_MONETIZABLE_PROVISIONAL",
    "CREATOR_STORE_PAGE",
    "DEVELOPER_SUPPORT_SERVER",
    "DISCOVERABLE",
    "FEATURABLE",
    "INVITES_DISABLED",
    "INVITE_SPLASH",
    "MEMBER_VERIFICATION_GATE_ENABLED",
    "MORE_STICKERS",
    "NEWS",
    "PARTNERED",
    "PREVIEW_ENABLED",
    "ROLE_ICONS",
    "ROLE_SUBSCRIPTIONS_AVAILABLE_FOR_PURCHAS",
    "ROLE_SUBSCRIPTIONS_ENABLED",
    "TICKETED_EVENTS_ENABLED",
    "VANITY_URL",
    "VERIFIED",
    "VIP_REGIONS",
    "WELCOME_SCREEN_ENABLED",
] as const;

export const integrationAccountSchema = z.object({
    id: z.string(),
    name: z.string(),
});

export const integrationApplicationSchema = z.object({
    id: z.string(),
    name: z.string(),
    icon: z.string().nullable(),
    description: z.string(),
    bot: userSchema.optional(),
});

export const integrationExpireBehaviors = [0, 1] as const;

export const applicationScopes = [
    "activities.read",
    "activities.write",
    "applications.builds.read",
    "applications.builds.upload",
    "applications.commands",
    "applications.commands.update",
    "applications.commands.permissions.update",
    "applications.entitlements",
    "applications.store.update",
    "bot",
    "connections",
    "dm_channels.read",
    "email",
    "gdm.join",
    "guilds",
    "guilds.join",
    "guilds.members.read",
    "identify",
    "messages.read",
    "relationships.read",
    "role_connections.write",
    "rpc",
    "rpc.activities.write",
    "rpc.notifications.read",
    "rpc.voice.read",
    "rpc.voice.write",
    "voice",
    "webhook.incoming",
] as const;

export const integrationSchema = z.object({
    id: z.string(),
    name: z.string(),
    type: z.string(),
    enabled: z.boolean(),
    syncing: z.boolean().optional(),
    role_id: z.boolean().optional(),
    enabled_emoticons: z.boolean().optional(),
    // use enum with number?
    // expire_behaviour: z.enum<number, number>(integrationExpireBehaviors).optional(),
    expire_behaviour: z.number().optional(),
    expire_grace_period: z.number().optional(),
    user: userSchema.optional(),
    account: integrationAccountSchema,
    synced_at: z.date().optional(),
    subscriber_count: z.number().optional(),
    revoked: z.boolean().optional(),
    application: integrationApplicationSchema,
    scopes: z.array(z.enum(applicationScopes)),
});

export const welcomeScreenChannelSchema = z.object({
    channel_id: z.string(),
    description: z.string(),
    emoji_id: z.string().nullable(),
    emoji_name: z.string().nullable(),
});

export const welcomeScreenSchema = z.object({
    description: z.string().nullable(),
    welcome_channels: z.array(welcomeScreenChannelSchema),
});

export const guildSchema = z.object({
    id: z.string(),
    name: z.string(),
    icon: z.string(),
    icon_hash: z.string().nullable(),
    splash: z.string().nullable(),
    discovery_splash: z.string().nullable(),
    owner: z.boolean().optional(),
    owner_id: z.string(),
    permissions: z.string().optional(),
    afk_channel_id: z.string().optional(),
    afk_timeout: z.number(),
    widget_enabled: z.boolean().optional(),
    widget_channel_id: z.string().nullish(),
    verification_level: z.number(),
    default_message_notifications: z.number(),
    explicit_content_filter: z.number(),
    roles: z.array(roleSchema),
    emojis: z.array(emojiSchema),
    features: z.array(z.enum(guildFeatures)),
    mfa_level: z.number(),
    application_id: z.string().nullable(),
    system_channel_id: z.string().nullable(),
    system_channel_flags: z.number(),
    rules_channel_id: z.string().optional(),
    max_presences: z.number().nullish(),
    max_members: z.number().optional(),
    vanity_url_code: z.string().nullable(),
    description: z.string().nullable(),
    banner: z.string().nullable(),
    premium_tuer: z.number(),
    premium_subscription_count: z.number().optional(),
    preferred_locale: z.string(),
    public_updates_channel_id: z.string().optional(),
    max_vidoe_channel_users: z.number().optional(),
    approximate_member_count: z.number().optional(),
    welcome_screen: welcomeScreenSchema,
    nsfw_level: z.number(),
    stickers: z.array(stickerSchema),
    premium_progress_bar_enabled: z.boolean(),
});

/**
 * Schema for "partial guild objects"
 * Used for Get Current User Guilds api
 */
export const guildPartialSchema = z.object({
    id: z.string(),
    name: z.string(),
    icon: z.string(),
    owner: z.boolean(),
    permissions: z.string().optional(),
    features: z.array(z.enum(guildFeatures)),
});

export type GuildSchema = z.infer<typeof guildSchema>;
export type GuildPartialSchema = z.infer<typeof guildPartialSchema>;

export const guildUserSchema = z.object({
    user: userSchema,
    nick: z.string().nullish(),
    avatar: z.string().nullish(),
    roles: z.array(z.string()),
    joined_at: z.date(),
    premium_since: z.date().nullish(),
    deaf: z.boolean(),
    mute: z.boolean(),
    pending: z.boolean().optional(),
    permissions: z.string(),
    communication_disabled_until: z.date().nullish(),
});

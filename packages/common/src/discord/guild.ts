import { z } from "zod";
import { emojiSchema } from "./emoji";
import { roleSchema } from "./role";
import { stickerSchema } from "./sticker";
import { userSchema } from "./user";

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

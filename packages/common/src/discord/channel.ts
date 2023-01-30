import { z } from "zod";
import { guildUserSchema } from "./guild";
import { userSchema } from "./user";

export const channelOverwriteSchema = z.object({
    id: z.string(),
    type: z.number(),
    allow: z.string(),
    deny: z.string(),
});

export const threadMetadataSchema = z.object({
    archived: z.boolean(),
    auto_archive_duration: z.number(),
    archive_timestamp: z.date(),
    locked: z.boolean(),
    invitable: z.boolean(),
    create_timestamp: z.date(),
});

export const threadMemberSchema = z.object({
    id: z.string().optional(),
    user_id: z.string().optional(),
    join_timestamp: z.date(),
    flags: z.number(),
    member: guildUserSchema.optional(),
});

export const tagSchema = z.object({
    id: z.string(),
    name: z.string(),
    moderated: z.boolean(),
    emoji_id: z.string().nullable(),
    emoji_name: z.string().nullable(),
});

export const channelSchema = z.object({
    id: z.string(),
    type: z.number(),
    guild_id: z.string().optional(),
    position: z.number().optional(),
    permission_overwrites: z.array(channelOverwriteSchema).optional(),
    name: z.string().nullish(),
    topic: z.string().nullish(),
    nsfw: z.boolean().optional(),
    last_message_id: z.string().nullable(),
    bitrate: z.number().optional(),
    user_limit: z.number().optional(),
    rate_limit_per_user: z.number().optional(),
    recipients: z.array(userSchema),
    icon: z.string().nullish(),
    owner_id: z.string().optional(),
    parent_id: z.string().nullish(),
    last_pin_timestamp: z.date().nullish(),
    rtc_region: z.string().nullish(),
    video_quality_mode: z.number(),
    message_count: z.number().optional(),
    member_count: z.number().optional(),
    thread_meta_data: threadMetadataSchema,
    member: threadMemberSchema,
    default_auto_archive_duration: z.number(),
    permission: z.string(),
    flags: z.number(),
    total_message_sent: z.number(),
    available_tags: z.array(tagSchema),
});

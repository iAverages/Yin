import { z } from "zod";

import { channelSchema } from "./channel";
import { guildSchema } from "./guild";
import { userSchema } from "./user";

export const webhookTypes = {
    1: "Incoming",
    2: "Channel Follower",
    3: "Application",
} as const;

const webhookTypesKey = [...Object.keys(webhookTypes)] as [string, ...string[]];

export const webhookSchema = z.object({
    id: z.string(),
    type: z.enum(webhookTypesKey),
    guild_id: z.string().nullish(),
    channel_id: z.string().nullable(),
    user: userSchema,
    name: z.string().nullable(),
    avatar: z.string().nullable(),
    token: z.string().optional(),
    application_id: z.string().nullable(),
    source_guild: guildSchema.partial().optional(),
    source_channel: channelSchema.partial().optional(),
    url: z.string().optional(),
});

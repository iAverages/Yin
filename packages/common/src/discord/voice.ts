import { z } from "zod";

import { guildUserSchema } from "./guild";

export const voiceStateSchema = z.object({
    guild_id: z.string().optional(),
    channel_id: z.string(),
    user_id: z.string(),
    member: guildUserSchema.optional(),
    session_id: z.string(),
    deaf: z.boolean(),
    mute: z.boolean(),
    self_deaf: z.boolean(),
    self_mute: z.boolean(),
    self_stream: z.boolean(),
    self_video: z.boolean(),
    suppress: z.boolean(),
    request_to_speak_timestamp: z.date(),
});

export const voiceRegionSchema = z.object({
    id: z.string(),
    name: z.string(),
    optimal: z.boolean(),
    deprecated: z.boolean(),
    custom: z.boolean(),
});

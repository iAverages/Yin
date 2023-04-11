import { z } from "zod";

import { userSchema } from "./user";

export const stickerSchema = z.object({
    id: z.string(),
    pack_id: z.string().optional(),
    name: z.string(),
    description: z.string().nullable(),
    tags: z.string().optional(),
    asset: z.string().optional(),
    type: z.number(),
    format_type: z.number(),
    available: z.boolean().optional(),
    guild_id: z.string().optional(),
    user: userSchema.optional(),
    sort_value: z.number(),
});

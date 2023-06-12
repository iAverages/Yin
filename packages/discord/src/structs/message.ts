import { z } from "zod";

import { userSchema } from "../structs/user";

export const messageSchema = z.object({
    id: z.string(),
    channel_id: z.string(),
    author: userSchema,
    content: z.string(),
    timestamp: z.string(),
    webhook_id: z.string().optional(),
});

export type Message = z.infer<typeof messageSchema>;

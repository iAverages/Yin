import { z } from "zod";

export const executeWebhoko = z.object({
    content: z.string(),
    username: z.string().optional(),
    avatar_url: z.string().optional(),
    tts: z.boolean().optional(),
});

export type ExecuteWebhookBody = z.infer<typeof executeWebhoko>;

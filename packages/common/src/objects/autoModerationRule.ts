import { z } from "zod";

// https://discord.com/developers/docs/resources/auto-moderation#auto-moderation-rule-object-auto-moderation-rule-structure
const autoModerationRuleSchema = z.object({
    id: z.string(),
    guild_id: z.string(),
    name: z.string(),
    creator_id: z.string(),
    event_type: z.number(),
    trigger_type: z.number(),
    trigger_metadata: z.unknown(),
    actions: z.array(z.object({})),
    enabled: z.boolean(),
    exempt_roles: z.array(z.string()),
    exempt_channels: z.array(z.string()),
});

export type AutoModerationRule = z.infer<typeof autoModerationRuleSchema>;

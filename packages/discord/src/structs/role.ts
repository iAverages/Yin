import { z } from "zod";

// Quote from the docs
// "Tags with type null represent booleans. They will be present and
// set to null if they are "true", and will be not present if they are "false"."
// https://discord.com/developers/docs/topics/permissions#role-object-role-tags-structure
const nullIsTrue = z.preprocess((val) => val === null, z.boolean());

export const roleTagSchema = z.object({
    bot_id: z.string().optional(),
    integration_id: z.string().optional(),
    premium_subscriber: nullIsTrue.optional(),
    subscription_listing_id: z.string().optional(),
    available_for_purchase: nullIsTrue.optional(),
    guild_connections: nullIsTrue.optional(),
});

export const roleSchema = z.object({
    id: z.string(),
    name: z.string(),
    color: z.number(),
    hoist: z.boolean(),
    icon: z.string().nullish(),
    unicode_emoji: z.string().nullish(),
    position: z.number(),
    permissions: z.string(),
    managed: z.boolean(),
    mentionable: z.boolean(),
    tags: roleTagSchema.optional(),
});

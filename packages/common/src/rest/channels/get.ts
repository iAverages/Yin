import { req } from "rest";
import { Http, Routes } from "rest/routes";
import { z } from "zod";

const auditLogChangeSchema = z.object({
    // "mixed (matches object field's type)" ???
    // https://discord.com/developers/docs/resources/audit-log#audit-log-change-object-audit-log-change-structure
    new_value: z.any().optional(),
    old_value: z.any().optional(),
    key: z.string(),
});

const auditLogEntrySchema = z.object({
    target_id: z.string(),
    changes: z.array(auditLogChangeSchema),
    user_id: z.string(),
    id: z.string(),
    action_type: z.number(),
    options: z.string().optional(),
    reason: z.string().optional(),
});

const responseSchema = z.object({
    application_commands: z.array(z.any()),
    audit_log_entries: z.array(auditLogEntrySchema),
    auto_moderation_rules: z.array(z.any()),
    guild_scheduled_events: z.array(z.any()),
    integrations: z.array(z.any()),
    threads: z.array(z.any()),
    users: z.array(z.any()),
    webhooks: z.array(z.any()),
});

export type GetChannel = z.infer<typeof responseSchema>;

export type GetChannelQueryParams = {
    user_id?: string;
    action_type?: number;
    before?: string;
    limit?: number;
};

export type GetChannelUrlParts = {
    "guild.id": string;
    queryParams?: GetChannelQueryParams;
};

export const get = (parts: GetChannelUrlParts) =>
    req({
        url: Routes.AUDIT_LOGS,
        method: Http.GET,
        schema: responseSchema,
        urlParts: parts,
    });

enum ChannelTypes {
    GUILD_TEXT = 0,
    DM = 1,
    GUILD_VOICE = 2,
    GROUP_DM = 3,
    GUILD_CATEGORY = 4,
    GUILD_ANNOUNCEMENT = 5,
    ANNOUNCEMENT_THREAD = 10,
    PUBLIC_THREAD = 11,
    PRIVATE_THREAD = 12,
    GUILD_STAGE_VOICE = 13,
    GUILD_DIRECTORY = 14,
    GUILD_FORUM = 15,
}

type ThreadTypes = ChannelTypes.PUBLIC_THREAD | ChannelTypes.PRIVATE_THREAD | ChannelTypes.ANNOUNCEMENT_THREAD;

type Channel = {
    id: string;
    type: Exclude<ChannelTypes, ThreadTypes>;
};

type Thread = {
    id: string;
    type: ThreadTypes;
    member: {};
};

type TextChannel = Channel | Thread;

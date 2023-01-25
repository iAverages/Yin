import { z } from "zod";

// TODO: https://discord.com/developers/docs/resources/audit-log#audit-log-entry-object-optional-audit-entry-info

export const auditLogSchema = z.object({
    application_commands: z.array(z.object({})),
    audit_log_entries: z.array(z.object({})),
    auto_moderation_rules: z.array(z.object({})),
    guild_scheduled_events: z.array(z.object({})),
    integrations: z.array(z.object({})),
    threads: z.array(z.object({})),
    users: z.array(z.object({})),
    webhooks: z.array(z.object({})),
});

export type AuditLog = z.infer<typeof auditLogSchema>;

export const auditLogEntrySchema = z.object({
    target_id: z.string().nullable(),
    changes: z.array(z.object({})).optional(),
    user_id: z.unknown(),
    id: z.string(),
    action_type: z.unknown(),
    options: z.unknown().optional(),
    reason: z.string().optional(),
});

export type AuditLogEntry = z.infer<typeof auditLogEntrySchema>;

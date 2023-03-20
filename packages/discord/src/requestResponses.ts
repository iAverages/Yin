// import { AuditLogs, AuditLogsUrlParts } from "./guilds/audit-logs";
import { Routes } from "./routes";

// prettier-ignore
export type RequestResponses<T> = 
    // T extends Routes.AUDIT_LOGS ? AuditLogs : 
    T extends Routes.CHANNEL ? {} : 
    null;

// prettier-ignore
export type UrlParts<T> = 
    // T extends Routes.AUDIT_LOGS ? AuditLogsUrlParts : 
    T extends Routes.CHANNEL ? {} : 
    null;

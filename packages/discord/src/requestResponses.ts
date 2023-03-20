// import { AuditLogs, AuditLogsUrlParts } from "./guilds/audit-logs";
import { GetUserUrlParts, User } from "./structs/user";
import { Routes } from "./routes";
import { GuildPartialSchema } from "structs/guild";

// prettier-ignore
export type RequestResponses<T> = 
    T extends Routes.USERS_ME ? User : 
    T extends Routes.USER ? User  : 
    T extends Routes.USERS_ME_GUILDS ? GuildPartialSchema : 
    null;

// prettier-ignore
export type UrlParts<T> = 
    T extends Routes.USER ? GetUserUrlParts :
    null;

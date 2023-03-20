export enum Routes {
    USERS_ME = "/users/@me",
    AUDIT_LOGS = "/guilds/{guild.id}/audit-logs",
    CHANNEL = "",
}

export const Http = ["GET", "POST", "PATCH", "DELETE"] as const;
// export type Http = typeof http;

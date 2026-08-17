import { queryOptions } from "@tanstack/solid-query";

import { apiFetch } from "./api-client";

export type AdminGuild = {
    id: string;
    name: string | null;
    icon: string | null;
    shardId: number;
};

export type BotRuntime = {
    instanceId: string;
    status: "reporting" | "stale" | "stopped";
    ageSeconds: number;
    snapshot: {
        botUserId: string | null;
        botName: string | null;
        environment: string;
        version: string;
        uptimeSeconds: number;
        shardCount: number | null;
        shards: { id: number; stage: string; latencyMs: number | null }[];
        cachedGuilds: number;
        unavailableGuilds: number;
        cachedUsers: number;
        cachedChannels: number;
    };
};

export const getAdminAccess = () => {
    return apiFetch<boolean>("/admin/access");
};

export const adminStateOptions = () => {
    return queryOptions({
        queryKey: ["admin", "state"],
        queryFn: () => apiFetch<BotRuntime[]>("/admin/state"),
        staleTime: 15_000,
        refetchInterval: 15_000,
    });
};

export const adminGuildsOptions = () => {
    return queryOptions({
        queryKey: ["admin", "guilds"],
        queryFn: () => apiFetch<AdminGuild[]>("/admin/guilds"),
        staleTime: 60_000,
        refetchInterval: 60_000,
    });
};

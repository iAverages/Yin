import type { AdminGuild, BotRuntime } from "../../src/lib/admin-queries.ts";

export const adminGuilds: AdminGuild[] = [
    { id: "9007199254740993", name: "Elsewhere", icon: null, shardId: 1 },
    { id: "103", name: "Night Shift", icon: null, shardId: 0 },
    { id: "999", name: null, icon: null, shardId: 1 },
];

export const adminRuntimes: BotRuntime[] = [
    {
        instanceId: "fixture-boot",
        ageSeconds: 2,
        status: "reporting",
        snapshot: {
            botUserId: "987654321012345678",
            botName: "Yin",
            environment: "production",
            version: "0.1.0",
            uptimeSeconds: 93784,
            shardCount: 2,
            shards: [
                { id: 0, stage: "connected", latencyMs: 42 },
                { id: 1, stage: "resuming", latencyMs: null },
            ],
            cachedGuilds: 2,
            unavailableGuilds: 1,
            cachedUsers: 123,
            cachedChannels: 15,
        },
    },
];

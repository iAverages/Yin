import { noop, type QueryClient, queryOptions } from "@tanstack/solid-query";

import { env } from "../env";
import { getGuildSettings, getManagedGuilds } from "./api-client";
import { waitForSsrData } from "./ssr-data";

export const managedGuildsOptions = () => {
    return queryOptions({
        queryKey: ["managed-guilds"],
        queryFn: getManagedGuilds,
        staleTime: 30_000,
    });
};

export const settingsQueryKey = (guildId: string) => {
    return ["guild-settings", guildId] as const;
};

export const guildSettingsOptions = (guildId: string) => {
    return queryOptions({
        queryKey: settingsQueryKey(guildId),
        queryFn: () => getGuildSettings(guildId),
        staleTime: 30_000,
    });
};

export const loadGuildSettings = async ({
    context,
    params,
}: {
    context: {
        queryClient: QueryClient;
        ssrDataDeadline: number;
    };
    params: { guildId: string };
}) => {
    if (import.meta.env.SSR) {
        await waitForSsrData(
            context.queryClient.query(guildSettingsOptions(params.guildId)).catch(noop),
            context.ssrDataDeadline,
        );
    }
};

export const guildInstallUrl = (guildId?: string) => {
    const url = new URL("/install/discord/guild", env.VITE_AUTH_URL);
    if (guildId) url.searchParams.set("guild_id", guildId);
    return url.toString();
};

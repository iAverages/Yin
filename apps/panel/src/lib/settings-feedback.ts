import { createMutation, useQueryClient } from "@tanstack/solid-query";
import { toast } from "solid-sonner";

import type { GuildSettings } from "./api-client";
import { settingsQueryKey } from "./guild-queries";

export const createSettingsMutation = (guildId: string, guildName: string) => {
    const queryClient = useQueryClient();
    const toastId = `guild-settings:${guildId}`;
    return <TInput>(mutationFn: (input: TInput) => Promise<GuildSettings>, message: string) =>
        createMutation(() => ({
            mutationFn,
            onSuccess: (settings: GuildSettings) => {
                queryClient.setQueryData(settingsQueryKey(guildId), settings);
                toast.success(message, { id: toastId, description: guildName, duration: 5000 });
            },
            onError: (error: unknown) => {
                toast.error(error instanceof Error ? error.message : "Request failed.", {
                    id: toastId,
                    description: guildName,
                    duration: 8000,
                });
            },
        }));
};

// Failures are already reported by the mutation's onError toast.
export const saveSettings = <TInput>(
    mutation: { mutateAsync: (input: TInput) => Promise<GuildSettings> },
    input: TInput,
) => mutation.mutateAsync(input).catch(() => undefined);

import { queryOptions } from "@tanstack/solid-query";

import { apiFetch } from "./api-client";

export type EmbedDisplay = "full" | "media-only";

export type UserSocialEmbedSetting = {
    platform: string;
    enabled: boolean;
    display: EmbedDisplay;
};

export type UserSettings = {
    socialEmbeds: UserSocialEmbedSetting[];
};

export const userSettingsOptions = () => {
    return queryOptions({
        queryKey: ["user-settings"],
        queryFn: () => apiFetch<UserSettings>("/me/settings"),
        staleTime: 30_000,
    });
};

export const updateUserSocialEmbeds = (socialEmbeds: UserSocialEmbedSetting[]) => {
    return apiFetch<UserSettings>("/me/settings/social", {
        method: "PUT",
        body: JSON.stringify({ socialEmbeds }),
    });
};

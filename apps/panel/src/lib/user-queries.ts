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
    translationLanguage: string | null;
};

export const userSettingsOptions = () => {
    return queryOptions({
        queryKey: ["user-settings"],
        queryFn: () => apiFetch<UserSettings>("/me/settings"),
        staleTime: 30_000,
    });
};

export const updateUserSocialEmbeds = (settings: UserSettings) => {
    return apiFetch<UserSettings>("/me/settings/social", {
        method: "PUT",
        body: JSON.stringify(settings),
    });
};

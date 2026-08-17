import { createIsomorphicFn } from "@tanstack/solid-start";
import { getRequestHeader } from "@tanstack/solid-start/server";

import { env } from "../env";
import { ApiRequestError } from "./api-error";

// Only initial SSR forwards the request cookie. Client navigation and mutations
// still fetch the API directly; this does not create a panel RPC/proxy endpoint.
const requestCookie = createIsomorphicFn()
    .server(() => getRequestHeader("cookie"))
    .client(() => undefined);

export type ManagedGuild = {
    id: string;
    name: string;
    icon: string | null;
    permissions: string;
    canManage: boolean;
    botInstalled: boolean;
};

export type SocialEmbedSetting = {
    platform: string;
    enabled: boolean;
};

export type CustomCommand = {
    name: string;
    response: string;
};

export type LadderRule = {
    id: number;
    warningThreshold: number;
    windowSeconds: number;
    action: "timeout" | "kick" | "ban";
    durationSeconds: number | null;
};

export type GuildSettings = {
    guild: ManagedGuild;
    commandPrefix: string | null;
    activePrefix: string;
    translationLanguage: string | null;
    activeTranslationLanguage: string;
    socialEmbeds: SocialEmbedSetting[];
    customCommands: CustomCommand[];
    ladderRules: LadderRule[];
};

export type GeneralSettingsInput = {
    commandPrefix: string | null;
    translationLanguage: string | null;
};

export type SocialEmbedsInput = {
    disabledPlatforms: string[];
};

export type LadderRuleInput = Omit<LadderRule, "id">;

export const getManagedGuilds = () => {
    return apiFetch<ManagedGuild[]>("/guilds");
};

const guildFetch = (guildId: string, path: string, method = "GET", input?: unknown) => {
    return apiFetch<GuildSettings>(`/guilds/${encodeURIComponent(guildId)}${path}`, {
        method,
        body: input === undefined ? undefined : JSON.stringify(input),
    });
};

export const getGuildSettings = (guildId: string) => guildFetch(guildId, "/settings");

export const updateGeneralSettings = (guildId: string, input: GeneralSettingsInput) =>
    guildFetch(guildId, "/settings/general", "PUT", input);

export const updateSocialEmbeds = (guildId: string, input: SocialEmbedsInput) =>
    guildFetch(guildId, "/settings/social", "PUT", input);

export const upsertCustomCommand = (guildId: string, input: CustomCommand) =>
    guildFetch(guildId, "/custom-commands", "POST", input);

export const deleteCustomCommand = (guildId: string, name: string) =>
    guildFetch(guildId, `/custom-commands/${encodeURIComponent(name)}`, "DELETE");

export const createLadderRule = (guildId: string, input: LadderRuleInput) =>
    guildFetch(guildId, "/ladder-rules", "POST", input);

export const updateLadderRule = (guildId: string, ruleId: number, input: LadderRuleInput) =>
    guildFetch(guildId, `/ladder-rules/${ruleId}`, "PUT", input);

export const deleteLadderRule = (guildId: string, ruleId: number) =>
    guildFetch(guildId, `/ladder-rules/${ruleId}`, "DELETE");

export const apiFetch = async <T>(path: string, init: RequestInit = {}) => {
    const headers = new Headers(init.headers);
    const cookie = requestCookie();
    if (cookie) headers.set("cookie", cookie);
    headers.set("accept", "application/json");
    if (init.body) {
        headers.set("content-type", "application/json");
    }

    const response = await fetch(new URL(path, env.VITE_API_URL), {
        ...init,
        headers,
        credentials: "include",
        redirect: "error",
        signal: init.signal ?? AbortSignal.timeout(10_000),
    });
    if (!response.ok) {
        const message = await response.text();
        throw new ApiRequestError(
            response.status,
            message || `API request failed with status ${response.status}`,
        );
    }

    return (await response.json()) as T;
};

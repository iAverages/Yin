import { createFileRoute } from "@tanstack/solid-router";
import { revalidateLogic } from "@tanstack/solid-form";
import { For, type Accessor } from "solid-js";
import { z } from "zod";

import { GuildSettingsContent } from "../components/guild-settings-content";
import { GuildPageSkeleton } from "../components/loading";
import { Button } from "../components/button";
import { useAppForm } from "../components/form";
import { Table } from "../components/table";
import {
    deleteCustomCommand,
    updateGeneralSettings,
    updateSocialEmbeds,
    upsertCustomCommand,
    type CustomCommand,
    type GeneralSettingsInput,
    type GuildSettings,
    type SocialEmbedsInput,
} from "../lib/api-client";
import { createSettingsMutation, saveSettings } from "../lib/settings-feedback";
import { loadGuildSettings } from "../lib/guild-queries";

const GuildSettingsPage = () => {
    const params = Route.useParams();
    return (
        <GuildSettingsContent guildId={params().guildId}>
            {(settings) => <SettingsEditor settings={settings} guildId={params().guildId} />}
        </GuildSettingsContent>
    );
};

const SettingsEditor = (props: { settings: Accessor<GuildSettings>; guildId: string }) => {
    const guildId = props.guildId;
    const settingsMutation = createSettingsMutation(guildId, props.settings().guild.name);

    const generalMutation = settingsMutation(
        (input: GeneralSettingsInput) => updateGeneralSettings(guildId, input),
        "General settings saved.",
    );
    const socialMutation = settingsMutation(
        (input: SocialEmbedsInput) => updateSocialEmbeds(guildId, input),
        "Social embed settings saved.",
    );
    const commandMutation = settingsMutation(
        (input: CustomCommand) => upsertCustomCommand(guildId, input),
        "Custom command saved.",
    );
    const removeCommandMutation = settingsMutation(
        (name: string) => deleteCustomCommand(guildId, name),
        "Custom command removed.",
    );

    const generalForm = useAppForm(() => ({
        defaultValues: generalValues(props.settings()),
        validationLogic: revalidateLogic(),
        validators: { onDynamic: generalSchema },
        onSubmit: async ({ value, formApi }) => {
            const settings = await saveSettings(generalMutation, {
                commandPrefix: blankToNull(value.commandPrefix),
                translationLanguage: blankToNull(value.translationLanguage),
            });
            if (settings) formApi.reset(generalValues(settings));
        },
    }));

    const socialForm = useAppForm(() => ({
        defaultValues: socialValues(props.settings()),
        onSubmit: async ({ value, formApi }) => {
            const settings = await saveSettings(socialMutation, {
                disabledPlatforms: Object.keys(value.enabled).filter(
                    (platform) => !value.enabled[platform],
                ),
            });
            if (settings) formApi.reset(socialValues(settings));
        },
    }));

    const commandForm = useAppForm(() => ({
        defaultValues: { name: "", response: "" },
        validationLogic: revalidateLogic(),
        validators: { onDynamic: commandSchema },
        onSubmit: async ({ value, formApi }) => {
            if (await saveSettings(commandMutation, value)) formApi.reset();
        },
    }));

    return (
        <div class="min-w-0">
            <header class="max-w-[760px]">
                <h1 class="m-0 text-2xl font-semibold tracking-tight">
                    {props.settings().guild.name}
                </h1>
                <p class="mt-2 mb-0 text-sm text-muted">Server settings</p>
            </header>

            <div class="mt-8 grid items-start gap-4 lg:grid-cols-[minmax(0,1fr)_340px]">
                <section
                    class="rounded-lg border border-line bg-surface p-5 md:p-6"
                    aria-labelledby="general-title"
                >
                    <SectionHeader title="General" id="general-title" />
                    <generalForm.AppForm>
                        <generalForm.Form>
                            <div class="mt-5 grid gap-4 sm:grid-cols-2">
                                <generalForm.AppField name="commandPrefix">
                                    {(field) => (
                                        <field.TextField
                                            label="Command prefix"
                                            description={`Leave blank to use ${props.settings().activePrefix === "!" ? "the default !" : "the bot default"}. No spaces.`}
                                            maxlength={16}
                                            placeholder="!"
                                        />
                                    )}
                                </generalForm.AppField>
                                <generalForm.AppField name="translationLanguage">
                                    {(field) => (
                                        <field.TextField
                                            label="Translation language"
                                            description="ISO code used by translated social embeds. Leave blank for English."
                                            placeholder="en, ja, pt-BR"
                                        />
                                    )}
                                </generalForm.AppField>
                            </div>
                            <div class="mt-5 flex items-center justify-between gap-3 border-t border-line pt-5">
                                <p class="m-0 text-xs text-muted">
                                    Active prefix:{" "}
                                    <span class="font-mono text-foreground">
                                        {props.settings().activePrefix}
                                    </span>{" "}
                                    - Language:{" "}
                                    <span class="font-mono text-foreground">
                                        {props.settings().activeTranslationLanguage}
                                    </span>
                                </p>
                                <generalForm.SubmitButton>Save general</generalForm.SubmitButton>
                            </div>
                        </generalForm.Form>
                    </generalForm.AppForm>
                </section>

                <section
                    class="rounded-lg border border-line bg-surface p-5 md:p-6"
                    aria-labelledby="social-title"
                >
                    <SectionHeader title="Social embeds" id="social-title" />
                    <socialForm.AppForm>
                        <socialForm.Form>
                            <div class="mt-2">
                                <For each={props.settings().socialEmbeds}>
                                    {(platform) => (
                                        <socialForm.AppField name={`enabled.${platform.platform}`}>
                                            {(field) => (
                                                <field.ToggleField
                                                    label={platformLabel(platform.platform)}
                                                    description="Replace links with embeds."
                                                />
                                            )}
                                        </socialForm.AppField>
                                    )}
                                </For>
                            </div>
                            <socialForm.SubmitButton class="mt-5 w-full">
                                Save social embeds
                            </socialForm.SubmitButton>
                        </socialForm.Form>
                    </socialForm.AppForm>
                </section>
            </div>

            <section
                class="mt-4 rounded-lg border border-line bg-surface p-5 md:p-6"
                aria-labelledby="commands-title"
            >
                <SectionHeader title="Custom commands" id="commands-title" />
                <div class="mt-5 grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
                    <commandForm.AppForm>
                        <commandForm.Form class="rounded-md border border-line bg-elevated/40 p-4">
                            <commandForm.AppField name="name">
                                {(field) => (
                                    <field.TextField
                                        label="Command name"
                                        description="Letters, numbers, dashes, and underscores only."
                                        placeholder="rules"
                                    />
                                )}
                            </commandForm.AppField>
                            <div class="mt-4">
                                <commandForm.AppField name="response">
                                    {(field) => (
                                        <field.TextareaField
                                            label="Response text"
                                            description="Yin sends this text when someone uses the command."
                                            maxlength={2000}
                                            placeholder="Read #start-here before posting."
                                        />
                                    )}
                                </commandForm.AppField>
                            </div>
                            <commandForm.SubmitButton class="mt-4 w-full">
                                Create or update command
                            </commandForm.SubmitButton>
                        </commandForm.Form>
                    </commandForm.AppForm>

                    <div class="overflow-hidden rounded-md border border-line">
                        <Table>
                            <thead>
                                <tr>
                                    <th>Name</th>
                                    <th>Response</th>
                                    <th />
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each={props.settings().customCommands}
                                    fallback={
                                        <tr>
                                            <td colspan={3}>No custom commands yet.</td>
                                        </tr>
                                    }
                                >
                                    {(command) => (
                                        <tr>
                                            <td>
                                                <span class="font-mono text-foreground">
                                                    {props.settings().activePrefix}
                                                    {command.name}
                                                </span>
                                            </td>
                                            <td>
                                                <span class="line-clamp-2">{command.response}</span>
                                            </td>
                                            <td>
                                                <Button
                                                    size="small"
                                                    variant="danger"
                                                    disabled={removeCommandMutation.isPending}
                                                    onClick={() =>
                                                        removeCommandMutation.mutate(command.name)
                                                    }
                                                >
                                                    Remove
                                                </Button>
                                            </td>
                                        </tr>
                                    )}
                                </For>
                            </tbody>
                        </Table>
                    </div>
                </div>
            </section>
        </div>
    );
};

const SectionHeader = (props: { title: string; id: string }) => {
    return (
        <h2 class="m-0 text-base font-semibold" id={props.id}>
            {props.title}
        </h2>
    );
};

const generalSchema = z.object({
    commandPrefix: z
        .string()
        .trim()
        .refine(
            (prefix) => prefix === "" || ([...prefix].length <= 16 && !/\s/.test(prefix)),
            "Prefixes must be at most 16 characters and contain no whitespace.",
        ),
    translationLanguage: z
        .string()
        .trim()
        .refine(
            (language) => language === "" || /^[a-z]{2,3}([-_][a-z]{2,4})?$/i.test(language),
            "Use a valid ISO language code such as en, ja, pt-BR, or zh-Hant.",
        ),
});

const commandSchema = z.object({
    name: z
        .string()
        .trim()
        .regex(
            /^[a-z0-9_-]{1,32}$/i,
            "Names must be 1-32 characters using only letters, numbers, `_`, or `-`.",
        ),
    response: z
        .string()
        .trim()
        .refine(
            (response) => response.length > 0 && [...response].length <= 2000,
            "Command response must be 1-2,000 characters.",
        ),
});

const generalValues = (settings: GuildSettings) => ({
    commandPrefix: settings.commandPrefix ?? "",
    translationLanguage: settings.translationLanguage ?? "",
});

const socialValues = (settings: GuildSettings) => ({
    enabled: Object.fromEntries(
        settings.socialEmbeds.map((platform) => [platform.platform, platform.enabled]),
    ) as Record<string, boolean>,
});

const blankToNull = (value: string) => {
    const trimmed = value.trim();
    return trimmed.length === 0 ? null : trimmed;
};

const platformLabel = (platform: string) => {
    const labels: Record<string, string> = {
        twitter: "Twitter / X",
        bluesky: "Bluesky",
        instagram: "Instagram",
        facebook: "Facebook",
        tiktok: "TikTok",
        spotify: "Spotify",
    };
    return labels[platform] ?? platform;
};

export const Route = createFileRoute("/_dashboard/guilds/$guildId")({
    context: ({ context }) => ({
        breadcrumbs: [
            ...context.breadcrumbs,
            { label: "Servers", to: "/" },
            { label: "Server settings" },
        ],
    }),
    remountDeps: ({ params }) => params.guildId,
    loader: loadGuildSettings,
    pendingComponent: GuildPageSkeleton,
    component: GuildSettingsPage,
    head: () => ({
        meta: [
            { title: "Server settings | Yin" },
            {
                name: "description",
                content: "Manage Yin settings for a Discord server.",
            },
        ],
    }),
});

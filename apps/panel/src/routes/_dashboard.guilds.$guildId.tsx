import { createFileRoute } from "@tanstack/solid-router";
import { For, createComputed, createSignal, type Accessor } from "solid-js";

import { GuildSettingsContent } from "../components/guild-settings-content";
import { GuildPageSkeleton } from "../components/loading";
import { Button } from "../components/button";
import { Field, FieldLabel, inputClass } from "../components/field";
import { Table } from "../components/table";
import { Toggle } from "../components/toggle";
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
import { createSettingsMutation } from "../lib/settings-feedback";
import { loadGuildSettings } from "../lib/guild-queries";
import { cn } from "../lib/utils";

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
    const [prefix, setPrefix] = createSignal("");
    const [language, setLanguage] = createSignal("");
    const [disabledPlatforms, setDisabledPlatforms] = createSignal<string[]>([]);
    const [commandName, setCommandName] = createSignal("");
    const [commandResponse, setCommandResponse] = createSignal("");

    createComputed(() => {
        const settings = props.settings();
        setPrefix(settings.commandPrefix ?? "");
        setLanguage(settings.translationLanguage ?? "");
        setDisabledPlatforms(
            settings.socialEmbeds
                .filter((platform) => !platform.enabled)
                .map((platform) => platform.platform),
        );
    });

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
        () => {
            setCommandName("");
            setCommandResponse("");
        },
    );
    const removeCommandMutation = settingsMutation(
        (name: string) => deleteCustomCommand(guildId, name),
        "Custom command removed.",
    );
    const saveGeneral = () => {
        generalMutation.mutate({
            commandPrefix: blankToNull(prefix()),
            translationLanguage: blankToNull(language()),
        });
    };

    const saveSocial = () => {
        socialMutation.mutate({ disabledPlatforms: disabledPlatforms() });
    };

    const saveCommand = () => {
        commandMutation.mutate({ name: commandName(), response: commandResponse() });
    };

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
                    <div class="mt-5 grid gap-4 sm:grid-cols-2">
                        <Field
                            label="Command prefix"
                            description={`Leave blank to use ${props.settings().activePrefix === "!" ? "the default !" : "the bot default"}. No spaces.`}
                            value={prefix()}
                            maxlength={16}
                            placeholder="!"
                            onInput={(event) => setPrefix(event.currentTarget.value)}
                        />
                        <Field
                            label="Translation language"
                            description="ISO code used by translated social embeds. Leave blank for English."
                            value={language()}
                            placeholder="en, ja, pt-BR"
                            onInput={(event) => setLanguage(event.currentTarget.value)}
                        />
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
                        <Button onClick={saveGeneral} disabled={generalMutation.isPending}>
                            Save general
                        </Button>
                    </div>
                </section>

                <section
                    class="rounded-lg border border-line bg-surface p-5 md:p-6"
                    aria-labelledby="social-title"
                >
                    <SectionHeader title="Social embeds" id="social-title" />
                    <div class="mt-2">
                        <For each={props.settings().socialEmbeds}>
                            {(platform) => (
                                <Toggle
                                    label={platformLabel(platform.platform)}
                                    description="Replace links with embeds."
                                    checked={!disabledPlatforms().includes(platform.platform)}
                                    onChange={(checked) => {
                                        setDisabledPlatforms((current) => {
                                            if (checked)
                                                return current.filter(
                                                    (item) => item !== platform.platform,
                                                );
                                            return Array.from(
                                                new Set([...current, platform.platform]),
                                            );
                                        });
                                    }}
                                />
                            )}
                        </For>
                    </div>
                    <Button
                        class="mt-5 w-full"
                        onClick={saveSocial}
                        disabled={socialMutation.isPending}
                    >
                        Save social embeds
                    </Button>
                </section>
            </div>

            <section
                class="mt-4 rounded-lg border border-line bg-surface p-5 md:p-6"
                aria-labelledby="commands-title"
            >
                <SectionHeader title="Custom commands" id="commands-title" />
                <div class="mt-5 grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
                    <div class="rounded-md border border-line bg-elevated/40 p-4">
                        <Field
                            label="Command name"
                            description="Letters, numbers, dashes, and underscores only."
                            value={commandName()}
                            placeholder="rules"
                            onInput={(event) => setCommandName(event.currentTarget.value)}
                        />
                        <FieldLabel
                            class="mt-4"
                            for="command-response"
                            label="Response text"
                            description="Yin sends this text when someone uses the command."
                        >
                            <textarea
                                id="command-response"
                                class={cn(inputClass, "min-h-32 resize-y bg-surface py-2")}
                                maxlength={2000}
                                value={commandResponse()}
                                placeholder="Read #start-here before posting."
                                onInput={(event) => setCommandResponse(event.currentTarget.value)}
                            />
                        </FieldLabel>
                        <Button
                            class="mt-4 w-full"
                            onClick={saveCommand}
                            disabled={commandMutation.isPending}
                        >
                            Create or update command
                        </Button>
                    </div>

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

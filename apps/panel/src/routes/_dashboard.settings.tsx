import { createMutation, createQuery, noop, useQueryClient } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, type Accessor } from "solid-js";
import { toast } from "solid-sonner";

import { useAppForm } from "../components/form";
import { Skeleton } from "../components/loading";
import { QueryBoundary } from "../components/query-boundary";
import { platformLabel } from "../lib/format";
import { saveSettings } from "../lib/settings-feedback";
import { waitForSsrData } from "../lib/ssr-data";
import {
    updateUserSocialEmbeds,
    userSettingsOptions,
    type EmbedDisplay,
    type UserSettings,
} from "../lib/user-queries";

const toastId = "user-settings";

const UserSettingsPage = () => {
    return (
        <div class="min-w-0">
            <header class="max-w-[760px]">
                <h1 class="m-0 text-2xl font-semibold tracking-tight">Settings</h1>
                <p class="mt-2 mb-0 text-sm text-muted">
                    Applies to your links and /embed in every server.
                </p>
            </header>
            <QueryBoundary
                queryKey={userSettingsOptions().queryKey}
                fallback={<UserSettingsSkeleton />}
            >
                <UserSettingsData />
            </QueryBoundary>
        </div>
    );
};

const UserSettingsData = () => {
    const query = createQuery(() => userSettingsOptions());
    return (
        <Show when={query.data}>{(settings) => <SocialEmbedsEditor settings={settings} />}</Show>
    );
};

const SocialEmbedsEditor = (props: { settings: Accessor<UserSettings> }) => {
    const queryClient = useQueryClient();
    const mutation = createMutation(() => ({
        mutationFn: updateUserSocialEmbeds,
        onSuccess: (settings: UserSettings) => {
            queryClient.setQueryData(userSettingsOptions().queryKey, settings);
            toast.success("Settings saved.", { id: toastId, duration: 5000 });
        },
        onError: (error: unknown) => {
            toast.error(error instanceof Error ? error.message : "Request failed.", {
                id: toastId,
                duration: 8000,
            });
        },
    }));

    const form = useAppForm(() => ({
        defaultValues: formValues(props.settings()),
        onSubmit: async ({ value, formApi }) => {
            const settings = await saveSettings(
                mutation,
                props.settings().socialEmbeds.map(({ platform, display }) => {
                    const mode = value.platforms[platform];
                    return mode === "off"
                        ? { platform, enabled: false, display }
                        : { platform, enabled: true, display: mode };
                }),
            );
            if (settings) formApi.reset(formValues(settings));
        },
    }));

    return (
        <form.AppForm>
            <form.Form>
                <section
                    class="mt-8 rounded-lg border border-line bg-surface p-5 md:p-6"
                    aria-labelledby="social-title"
                >
                    <h2 class="m-0 text-base font-semibold" id="social-title">
                        Social embeds
                    </h2>
                    <p class="mt-2 mb-0 text-xs text-muted">
                        How Yin embeds your links. Media only sends just the images and videos.
                        Server settings can also turn a platform off, and the /embed display option
                        overrides this for one post.
                    </p>
                    <div class="mt-5 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        <For each={props.settings().socialEmbeds}>
                            {(platform) => (
                                <form.AppField name={`platforms.${platform.platform}`}>
                                    {(field) => (
                                        <field.SelectField label={platformLabel(platform.platform)}>
                                            <For each={modes}>
                                                {(mode) => (
                                                    <option
                                                        value={mode.value}
                                                        selected={
                                                            field().state.value === mode.value
                                                        }
                                                    >
                                                        {mode.label}
                                                    </option>
                                                )}
                                            </For>
                                        </field.SelectField>
                                    )}
                                </form.AppField>
                            )}
                        </For>
                    </div>
                    <div class="mt-5 flex justify-end">
                        <form.SubmitButton>Save settings</form.SubmitButton>
                    </div>
                </section>
            </form.Form>
        </form.AppForm>
    );
};

const UserSettingsSkeleton = () => {
    return (
        <div
            class="mt-8 space-y-5 rounded-lg border border-line bg-surface p-5 md:p-6"
            role="status"
            aria-label="Loading settings"
            aria-busy="true"
        >
            <Skeleton class="h-4 w-32" />
            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <For each={Array.from({ length: 6 })}>{() => <Skeleton class="h-14 w-full" />}</For>
            </div>
        </div>
    );
};

type Mode = EmbedDisplay | "off";

const modes: { value: Mode; label: string }[] = [
    { value: "off", label: "Off" },
    { value: "full", label: "Full" },
    { value: "media-only", label: "Media only" },
];

const formValues = (settings: UserSettings) => ({
    platforms: Object.fromEntries(
        settings.socialEmbeds.map(({ platform, enabled, display }) => [
            platform,
            enabled ? display : "off",
        ]),
    ) as Record<string, Mode>,
});

export const Route = createFileRoute("/_dashboard/settings")({
    context: ({ context }) => ({
        breadcrumbs: [...context.breadcrumbs, { label: "Settings" }],
    }),
    loader: async ({ context }) => {
        if (import.meta.env.SSR) {
            await waitForSsrData(
                context.queryClient.query(userSettingsOptions()).catch(noop),
                context.ssrDataDeadline,
            );
        }
    },
    component: UserSettingsPage,
    head: () => ({
        meta: [
            { title: "Settings | Yin" },
            { name: "description", content: "Manage your personal Yin settings." },
        ],
    }),
});

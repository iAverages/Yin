import { createFileRoute } from "@tanstack/solid-router";
import { revalidateLogic } from "@tanstack/solid-form";
import { For, Show, type Accessor } from "solid-js";
import { z } from "zod";

import { Badge } from "../components/badge";
import { Button } from "../components/button";
import { useAppForm } from "../components/form";
import { GuildSettingsContent } from "../components/guild-settings-content";
import { GuildPageSkeleton } from "../components/loading";
import { Table } from "../components/table";
import {
    createLadderRule,
    deleteLadderRule,
    type GuildSettings,
    type LadderRuleInput,
} from "../lib/api-client";
import { createSettingsMutation, saveSettings } from "../lib/settings-feedback";
import { loadGuildSettings } from "../lib/guild-queries";
import { formatDuration } from "../lib/format";

const ModerationPage = () => {
    const params = Route.useParams();
    return (
        <GuildSettingsContent guildId={params().guildId} moderation>
            {(settings) => <ModerationEditor settings={settings} guildId={params().guildId} />}
        </GuildSettingsContent>
    );
};

const ModerationEditor = (props: { settings: Accessor<GuildSettings>; guildId: string }) => {
    const guildId = props.guildId;
    const settingsMutation = createSettingsMutation(guildId, props.settings().guild.name);

    const ladderMutation = settingsMutation(
        (input: LadderRuleInput) => createLadderRule(guildId, input),
        "Punishment ladder rule added.",
    );
    const removeRuleMutation = settingsMutation(
        (ruleId: number) => deleteLadderRule(guildId, ruleId),
        "Punishment ladder rule removed.",
    );

    const ladderForm = useAppForm(() => ({
        defaultValues: {
            threshold: "3",
            windowDays: "30",
            action: "timeout" as LadderRuleInput["action"],
            timeoutDays: "1",
        },
        validationLogic: revalidateLogic(),
        validators: { onDynamic: ladderSchema },
        onSubmit: async ({ value }) => {
            await saveSettings(ladderMutation, {
                warningThreshold: Number.parseInt(value.threshold, 10),
                windowSeconds: Number.parseInt(value.windowDays, 10) * 86_400,
                action: value.action,
                durationSeconds:
                    value.action === "timeout"
                        ? Number.parseInt(value.timeoutDays, 10) * 86_400
                        : null,
            });
        },
    }));
    const action = ladderForm.useSelector((state) => state.values.action);

    return (
        <div class="min-w-0">
            <header>
                <h1 class="m-0 text-2xl font-semibold tracking-tight">
                    {props.settings().guild.name}
                </h1>
                <p class="mt-2 mb-0 text-sm text-muted">Moderation</p>
            </header>
            <section
                class="mt-8 rounded-lg border border-line bg-surface p-5 md:p-6"
                aria-labelledby="ladder-title"
            >
                <h2 class="m-0 text-base font-semibold" id="ladder-title">
                    Punishment ladder
                </h2>
                <div class="mt-5 grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
                    <ladderForm.AppForm>
                        <ladderForm.Form class="rounded-md border border-line bg-elevated/40 p-4">
                            <div class="grid grid-cols-2 gap-3">
                                <ladderForm.AppField name="threshold">
                                    {(field) => (
                                        <field.TextField label="Warnings" type="number" min="1" />
                                    )}
                                </ladderForm.AppField>
                                <ladderForm.AppField name="windowDays">
                                    {(field) => (
                                        <field.TextField
                                            label="Window days"
                                            type="number"
                                            min="1"
                                        />
                                    )}
                                </ladderForm.AppField>
                            </div>
                            <div class="mt-4">
                                <ladderForm.AppField name="action">
                                    {(field) => (
                                        <field.SelectField label="Action">
                                            <option value="timeout">Timeout</option>
                                            <option value="kick">Kick</option>
                                            <option value="ban">Ban</option>
                                        </field.SelectField>
                                    )}
                                </ladderForm.AppField>
                            </div>
                            <Show when={action() === "timeout"}>
                                <div class="mt-4">
                                    <ladderForm.AppField name="timeoutDays">
                                        {(field) => (
                                            <field.TextField
                                                label="Timeout days"
                                                type="number"
                                                min="1"
                                                max="28"
                                            />
                                        )}
                                    </ladderForm.AppField>
                                </div>
                            </Show>
                            <ladderForm.SubmitButton class="mt-4 w-full">
                                Add ladder rule
                            </ladderForm.SubmitButton>
                        </ladderForm.Form>
                    </ladderForm.AppForm>

                    <div class="overflow-hidden rounded-md border border-line">
                        <Table>
                            <thead>
                                <tr>
                                    <th>Trigger</th>
                                    <th>Action</th>
                                    <th />
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each={props.settings().ladderRules}
                                    fallback={
                                        <tr>
                                            <td colspan={3}>
                                                No punishment ladder rules configured.
                                            </td>
                                        </tr>
                                    }
                                >
                                    {(rule) => (
                                        <tr>
                                            <td>
                                                {rule.warningThreshold} warnings in{" "}
                                                {formatDuration(rule.windowSeconds)}
                                            </td>
                                            <td>
                                                <Badge
                                                    tone={
                                                        rule.action === "ban"
                                                            ? "danger"
                                                            : rule.action === "kick"
                                                              ? "warning"
                                                              : "neutral"
                                                    }
                                                >
                                                    {rule.action}
                                                    {rule.durationSeconds
                                                        ? ` - ${formatDuration(rule.durationSeconds)}`
                                                        : ""}
                                                </Badge>
                                            </td>
                                            <td>
                                                <Button
                                                    size="small"
                                                    variant="danger"
                                                    disabled={removeRuleMutation.isPending}
                                                    onClick={() =>
                                                        removeRuleMutation.mutate(rule.id)
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

const isWholeNumber = (value: string, max = Infinity) => {
    const days = Number(value);
    return Number.isInteger(days) && days >= 1 && days <= max;
};

const ladderSchema = z
    .object({
        threshold: z
            .string()
            .refine(isWholeNumber, "Warnings must be a whole number of at least 1."),
        windowDays: z.string().refine(isWholeNumber, "Window must be a whole number of days."),
        action: z.enum(["timeout", "kick", "ban"]),
        timeoutDays: z.string(),
    })
    .refine((rule) => rule.action !== "timeout" || isWholeNumber(rule.timeoutDays, 28), {
        path: ["timeoutDays"],
        error: "Timeout must be 1-28 days.",
    });

export const Route = createFileRoute("/_dashboard/guilds/$guildId_/moderation")({
    context: ({ context }) => ({
        breadcrumbs: [
            ...context.breadcrumbs,
            { label: "Servers", to: "/" },
            { label: "Moderation" },
        ],
    }),
    remountDeps: ({ params }) => params.guildId,
    loader: loadGuildSettings,
    pendingComponent: () => <GuildPageSkeleton moderation />,
    component: ModerationPage,
    head: () => ({ meta: [{ title: "Moderation | Yin" }] }),
});

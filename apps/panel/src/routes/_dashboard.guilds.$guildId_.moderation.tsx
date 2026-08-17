import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, createSignal, type Accessor } from "solid-js";

import { Badge } from "../components/badge";
import { Button } from "../components/button";
import { Field } from "../components/field";
import { GuildSettingsContent } from "../components/guild-settings-content";
import { GuildPageSkeleton } from "../components/loading";
import { SelectField } from "../components/select-field";
import { Table } from "../components/table";
import {
    createLadderRule,
    deleteLadderRule,
    type GuildSettings,
    type LadderRuleInput,
} from "../lib/api-client";
import { createSettingsMutation } from "../lib/settings-feedback";
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
    const [threshold, setThreshold] = createSignal("3");
    const [windowDays, setWindowDays] = createSignal("30");
    const [action, setAction] = createSignal<"timeout" | "kick" | "ban">("timeout");
    const [timeoutDays, setTimeoutDays] = createSignal("1");

    const ladderMutation = settingsMutation(
        (input: LadderRuleInput) => createLadderRule(guildId, input),
        "Punishment ladder rule added.",
    );
    const removeRuleMutation = settingsMutation(
        (ruleId: number) => deleteLadderRule(guildId, ruleId),
        "Punishment ladder rule removed.",
    );

    const addRule = () => {
        ladderMutation.mutate({
            warningThreshold: Number.parseInt(threshold(), 10),
            windowSeconds: Number.parseInt(windowDays(), 10) * 86_400,
            action: action(),
            durationSeconds:
                action() === "timeout" ? Number.parseInt(timeoutDays(), 10) * 86_400 : null,
        });
    };

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
                    <div class="rounded-md border border-line bg-elevated/40 p-4">
                        <div class="grid grid-cols-2 gap-3">
                            <Field
                                label="Warnings"
                                type="number"
                                min="1"
                                value={threshold()}
                                onInput={(event) => setThreshold(event.currentTarget.value)}
                            />
                            <Field
                                label="Window days"
                                type="number"
                                min="1"
                                value={windowDays()}
                                onInput={(event) => setWindowDays(event.currentTarget.value)}
                            />
                        </div>
                        <div class="mt-4">
                            <SelectField
                                label="Action"
                                value={action()}
                                onChange={(event) =>
                                    setAction(
                                        event.currentTarget.value as "timeout" | "kick" | "ban",
                                    )
                                }
                            >
                                <option value="timeout">Timeout</option>
                                <option value="kick">Kick</option>
                                <option value="ban">Ban</option>
                            </SelectField>
                        </div>
                        <Show when={action() === "timeout"}>
                            <div class="mt-4">
                                <Field
                                    label="Timeout days"
                                    type="number"
                                    min="1"
                                    max="28"
                                    value={timeoutDays()}
                                    onInput={(event) => setTimeoutDays(event.currentTarget.value)}
                                />
                            </div>
                        </Show>
                        <Button
                            class="mt-4 w-full"
                            onClick={addRule}
                            disabled={ladderMutation.isPending}
                        >
                            Add ladder rule
                        </Button>
                    </div>

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

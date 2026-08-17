import { createQuery, noop } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import { For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";

import { Badge } from "../components/badge";
import { Button } from "../components/button";
import { Field } from "../components/field";
import { QueryBoundary, QueryError } from "../components/query-boundary";
import { Table } from "../components/table";
import { adminGuildsOptions, adminStateOptions, type BotRuntime } from "../lib/admin-queries";
import { formatDuration } from "../lib/format";
import { waitForSsrData } from "../lib/ssr-data";

const Loading = (props: { label: string }) => {
    return (
        <p role="status" class="rounded-lg border border-line p-5 text-muted">
            Loading {props.label}...
        </p>
    );
};

const AdminPage = () => {
    return (
        <div class="space-y-8">
            <header>
                <p class="mt-0 mb-2 font-mono text-xs text-muted">YIN / DIAGNOSTICS</p>
                <h1 class="m-0 text-2xl font-semibold tracking-tight">Bot overview</h1>
                <p class="mt-2 mb-0 text-sm text-muted">
                    Read-only runtime and guild directory. Gateway snapshots update every 15
                    seconds.
                </p>
            </header>
            <section aria-label="Gateway & runtime">
                <QueryBoundary
                    queryKey={adminStateOptions().queryKey}
                    fallback={<Loading label="bot state" />}
                >
                    <RuntimeState />
                </QueryBoundary>
            </section>
            <section aria-label="All guilds">
                <QueryBoundary
                    queryKey={adminGuildsOptions().queryKey}
                    fallback={<Loading label="guilds" />}
                >
                    <GuildDirectory />
                </QueryBoundary>
            </section>
        </div>
    );
};

const StaleError = (props: {
    query: { error: Error | null; isFetching: boolean; refetch: () => Promise<unknown> };
    message: string;
}) => {
    return (
        <Show when={props.query.error}>
            <div class="mb-4">
                <p class="text-sm text-warning">Refresh failed. {props.message}</p>
                <QueryError
                    error={props.query.error}
                    pending={props.query.isFetching}
                    onRetry={() => void props.query.refetch()}
                />
            </div>
        </Show>
    );
};

const RuntimeState = () => {
    const state = createQuery(() => adminStateOptions());
    const [now, setNow] = createSignal(0);
    onMount(() => {
        setNow(Date.now());
        const timer = window.setInterval(() => setNow(Date.now()), 1000);
        onCleanup(() => window.clearInterval(timer));
    });
    const age = (runtime: BotRuntime) =>
        runtime.ageSeconds + Math.max(0, Math.floor((now() - state.dataUpdatedAt) / 1000));

    return (
        <>
            <div class="mb-4 flex items-center justify-between gap-3">
                <h2 id="runtime-title" class="m-0 text-lg font-semibold">
                    Gateway & runtime
                </h2>
                <Button
                    size="small"
                    variant="ghost"
                    disabled={state.isFetching}
                    onClick={() => void state.refetch()}
                >
                    Refresh state
                </Button>
            </div>
            <Show when={state.data}>
                <StaleError query={state} message="Displayed diagnostics may be out of date." />
                <div class="space-y-4">
                    <For
                        each={state.data}
                        fallback={
                            <p class="rounded-lg border border-line p-5 text-muted">
                                No bot snapshots in the last 24 hours. Check that the bot is running
                                and can write diagnostics to the database.
                            </p>
                        }
                    >
                        {(runtime) => <RuntimeCard runtime={runtime} ageSeconds={age(runtime)} />}
                    </For>
                </div>
                <p class="mt-3 mb-0 text-xs leading-relaxed text-muted">
                    Reporting means the process is publishing snapshots, not that its gateway is
                    healthy. After 45 seconds without a snapshot, an instance is stale. Stopped and
                    stale instances remain here for 24 hours.
                </p>
            </Show>
        </>
    );
};

const RuntimeCard = (props: { runtime: BotRuntime; ageSeconds: number }) => {
    const snapshot = () => props.runtime.snapshot;
    const status = () =>
        props.runtime.status === "reporting" && props.ageSeconds >= 45
            ? "stale"
            : props.runtime.status;
    const current = () => status() === "reporting";
    const connected = () => snapshot().shards.filter((shard) => shard.stage === "connected").length;
    return (
        <article
            class="overflow-hidden rounded-lg border border-line bg-elevated/40"
            aria-label={`Bot instance ${props.runtime.instanceId}`}
        >
            <div class="flex flex-wrap items-start justify-between gap-3 border-b border-line p-4">
                <div class="min-w-0">
                    <h3 class="m-0 text-base font-medium">
                        {snapshot().botName ?? "Awaiting Discord identity"}
                    </h3>
                    <p class="mt-1 mb-0 break-all font-mono text-xs text-muted">
                        {snapshot().botUserId ?? "No READY received"}
                    </p>
                </div>
                <div class="flex flex-wrap items-center gap-2">
                    <Badge
                        tone={current() ? "success" : status() === "stale" ? "warning" : "neutral"}
                    >
                        {status() === "reporting"
                            ? "Reporting"
                            : status() === "stale"
                              ? "Stale"
                              : "Stopped"}
                    </Badge>
                    <span class="font-mono text-xs text-muted">
                        Snapshot {formatDuration(props.ageSeconds, "narrow")} ago
                    </span>
                </div>
            </div>
            <Show when={!current()}>
                <p class="m-0 border-b border-line bg-warning/[.06] px-4 py-3 text-sm text-warning">
                    Historical snapshot - gateway statuses below are last known, not live.
                </p>
            </Show>
            <dl class="m-0 grid grid-cols-2 gap-5 p-4 sm:grid-cols-4">
                <Metric
                    label="Uptime at snapshot"
                    value={formatDuration(snapshot().uptimeSeconds, "narrow")}
                />
                <Metric label="Total shards" value={snapshot().shardCount ?? "Unknown"} />
                <Metric
                    label="Connected runners"
                    value={`${connected()} / ${snapshot().shards.length}`}
                />
                <Metric
                    label="Environment / version"
                    value={`${snapshot().environment} / ${snapshot().version}`}
                />
            </dl>
            <Table aria-label={`Shard statuses for ${props.runtime.instanceId}`}>
                <thead>
                    <tr>
                        <th scope="col">Shard</th>
                        <th scope="col">{current() ? "Gateway status" : "Last gateway status"}</th>
                        <th scope="col">Heartbeat latency</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each={snapshot().shards}
                        fallback={
                            <tr>
                                <td colSpan={3}>No shard runners reported yet.</td>
                            </tr>
                        }
                    >
                        {(shard) => (
                            <tr>
                                <td>
                                    <span class="font-mono">{shard.id}</span>
                                </td>
                                <td>
                                    <Badge
                                        tone={
                                            !current()
                                                ? "neutral"
                                                : shard.stage === "connected"
                                                  ? "success"
                                                  : shard.stage === "disconnected"
                                                    ? "danger"
                                                    : "warning"
                                        }
                                    >
                                        {shard.stage}
                                    </Badge>
                                </td>
                                <td>
                                    <span class="font-mono">
                                        {shard.latencyMs === null
                                            ? "Not measured"
                                            : `${shard.latencyMs} ms`}
                                    </span>
                                </td>
                            </tr>
                        )}
                    </For>
                </tbody>
            </Table>
            <dl class="m-0 grid grid-cols-2 gap-5 border-t border-line p-4 sm:grid-cols-4">
                <Metric label="Cached guilds" value={snapshot().cachedGuilds} />
                <Metric label="Unavailable guilds" value={snapshot().unavailableGuilds} />
                <Metric label="Cached users" value={snapshot().cachedUsers} />
                <Metric label="Cached channels" value={snapshot().cachedChannels} />
            </dl>
            <p class="m-0 border-t border-line px-4 py-3 text-xs text-muted">
                Process boot <span class="break-all font-mono">{props.runtime.instanceId}</span>
            </p>
        </article>
    );
};

const Metric = (props: { label: string; value: string | number }) => {
    return (
        <div>
            <dt class="text-xs text-muted">{props.label}</dt>
            <dd class="mt-1 ml-0 break-words font-mono text-sm">{props.value}</dd>
        </div>
    );
};

const GuildDirectory = () => {
    const guilds = createQuery(() => adminGuildsOptions());
    const [search, setSearch] = createSignal("");
    const filtered = createMemo(() => {
        const query = search().trim().toLowerCase();
        return (guilds.data ?? []).filter(
            (guild) => guild.id.includes(query) || guild.name?.toLowerCase().includes(query),
        );
    });
    return (
        <>
            <div class="mb-2 flex items-center justify-between gap-3">
                <h2 id="guild-directory-title" class="m-0 text-lg font-semibold">
                    All guilds
                </h2>
                <Button
                    size="small"
                    variant="ghost"
                    disabled={guilds.isFetching}
                    onClick={() => void guilds.refetch()}
                >
                    Refresh guilds
                </Button>
            </div>
            <p class="mt-0 mb-4 text-sm text-muted">
                All current bot memberships, including guilds you do not manage. Synced from gateway
                events; this directory is retained while the bot is offline.
            </p>
            <Show when={guilds.data}>
                <StaleError query={guilds} message="The directory may be out of date." />
                <div class="mb-4 max-w-md">
                    <Field
                        label="Search guilds"
                        placeholder="Name or Discord ID"
                        type="search"
                        value={search()}
                        onInput={(event) => setSearch(event.currentTarget.value)}
                    />
                </div>
                <p class="text-xs text-muted" role="status">
                    {filtered().length} of {guilds.data?.length ?? 0} guilds
                </p>
                <div class="overflow-hidden rounded-lg border border-line">
                    <Table aria-label="Bot guild directory">
                        <thead>
                            <tr>
                                <th scope="col">Guild</th>
                                <th scope="col">Discord ID</th>
                                <th scope="col">Shard</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each={filtered()}
                                fallback={
                                    <tr>
                                        <td colSpan={3}>
                                            {guilds.data?.length
                                                ? "No guilds match your search."
                                                : "No guild memberships recorded. Check the bot's gateway connection."}
                                        </td>
                                    </tr>
                                }
                            >
                                {(guild) => (
                                    <tr>
                                        <td>
                                            <span class="font-medium text-foreground">
                                                {guild.name ?? "Metadata pending"}
                                            </span>
                                        </td>
                                        <td>
                                            <span class="font-mono">{guild.id}</span>
                                        </td>
                                        <td>
                                            <span class="font-mono">{guild.shardId}</span>
                                        </td>
                                    </tr>
                                )}
                            </For>
                        </tbody>
                    </Table>
                </div>
            </Show>
        </>
    );
};

export const Route = createFileRoute("/_dashboard/admin/")({
    loader: async ({ context }) => {
        if (import.meta.env.SSR) {
            await waitForSsrData(
                Promise.all([
                    context.queryClient.query(adminStateOptions()).catch(noop),
                    context.queryClient.query(adminGuildsOptions()).catch(noop),
                ]),
                context.ssrDataDeadline,
            );
        }
    },
    component: AdminPage,
});

import { createQuery } from "@tanstack/solid-query";
import { Link, createFileRoute } from "@tanstack/solid-router";
import { For, Show } from "solid-js";

import { Button } from "../components/button";
import { Icon } from "../components/icon";
import { ServersSkeleton } from "../components/loading";
import { QueryBoundary } from "../components/query-boundary";
import { GuildMark } from "../components/identity-mark";
import type { ManagedGuild } from "../lib/api-client";
import { guildInstallUrl, managedGuildsOptions } from "../lib/guild-queries";

const DashboardPage = () => {
    return (
        <QueryBoundary queryKey={managedGuildsOptions().queryKey} fallback={<ServersSkeleton />}>
            <Dashboard />
        </QueryBoundary>
    );
};

const Dashboard = () => {
    const guilds = createQuery(() => managedGuildsOptions());

    return (
        <section aria-labelledby="servers-title">
            <div class="flex items-center justify-between gap-4">
                <h1 class="m-0 text-2xl font-semibold tracking-tight" id="servers-title">
                    Servers
                </h1>
                <Button
                    size="small"
                    variant="ghost"
                    disabled={guilds.isFetching}
                    onClick={() => guilds.refetch()}
                >
                    Refresh servers
                </Button>
            </div>
            <p class="mt-2 mb-6 text-sm text-muted">Select a server to edit its settings.</p>
            <Show when={guilds.data}>
                <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
                    <For
                        each={guilds.data}
                        fallback={
                            <p class="text-sm text-muted md:col-span-2">
                                No servers found. You need Manage Server or Administrator
                                permission.
                            </p>
                        }
                    >
                        {(guild) => <GuildCard guild={guild} />}
                    </For>
                </div>
            </Show>
        </section>
    );
};

const GuildCard = (props: { guild: ManagedGuild }) => {
    return (
        <Show
            when={props.guild.botInstalled}
            fallback={
                <a
                    class="flex min-w-0 items-center gap-3 rounded-lg border border-line/60 bg-canvas p-4 text-muted no-underline transition-colors hover:border-muted/50 hover:bg-surface"
                    href={guildInstallUrl(props.guild.id)}
                    aria-label={`Add Yin to ${props.guild.name}`}
                >
                    <GuildMark guild={props.guild} class="size-10 brightness-75 grayscale" />
                    <span class="min-w-0 flex-1">
                        <strong class="block truncate text-sm font-medium">
                            {props.guild.name}
                        </strong>
                        <span class="mt-1 block text-xs">Add Yin</span>
                    </span>
                    <Icon name="plus" size={16} />
                </a>
            }
        >
            <Link
                class="flex min-w-0 items-center gap-3 rounded-lg border border-line bg-elevated p-4 text-foreground no-underline transition-colors hover:border-accent/60 hover:bg-accent/10"
                to="/guilds/$guildId"
                params={{ guildId: props.guild.id }}
                aria-label={`Manage ${props.guild.name}`}
            >
                <GuildMark guild={props.guild} class="size-10" />
                <strong class="min-w-0 flex-1 truncate text-sm font-medium">
                    {props.guild.name}
                </strong>
                <span class="text-muted">
                    <Icon name="arrow-right" size={16} />
                </span>
            </Link>
        </Show>
    );
};

export const Route = createFileRoute("/_dashboard/")({
    context: ({ context }) => ({ breadcrumbs: [...context.breadcrumbs, { label: "Servers" }] }),
    pendingComponent: ServersSkeleton,
    component: DashboardPage,
    head: () => ({ meta: [{ title: "Servers | Yin" }] }),
});

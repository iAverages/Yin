import { Popover } from "@kobalte/core/popover";
import { createQuery } from "@tanstack/solid-query";
import { Link, useLocation, useParams } from "@tanstack/solid-router";
import { For, Show, createMemo, createSignal, type JSX } from "solid-js";

import { managedGuildsOptions } from "../lib/guild-queries";
import { Button } from "./button";
import { Icon } from "./icon";
import { GuildMark } from "./identity-mark";

export const GuildSelector = (props: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    mobile?: boolean;
    portalMount?: HTMLElement;
    onNavigate?: () => void;
}) => {
    const params = useParams({ strict: false });
    const location = useLocation();
    const guilds = createQuery(() => ({ ...managedGuildsOptions(), throwOnError: false }));
    const [search, setSearch] = createSignal("");
    const installed = createMemo(() => (guilds.data ?? []).filter((guild) => guild.botInstalled));
    const selected = () => installed().find((guild) => guild.id === params().guildId);
    const matches = createMemo(() => {
        const term = search().trim().toLocaleLowerCase();
        return installed().filter((guild) => guild.name.toLocaleLowerCase().includes(term));
    });
    let input: HTMLInputElement | undefined;
    let content: HTMLDivElement | undefined;

    const changeOpen = (open: boolean) => {
        if (open) setSearch("");
        props.onOpenChange(open);
    };
    const navigate = () => {
        props.onOpenChange(false);
        props.onNavigate?.();
    };
    const moveFocus: JSX.EventHandler<HTMLDivElement, KeyboardEvent> = (event) => {
        if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
        const links = Array.from(
            content?.querySelectorAll<HTMLAnchorElement>("a[data-guild-option]") ?? [],
        );
        if (!links.length) return;
        const current = links.findIndex((link) => link === document.activeElement);
        const next =
            current < 0
                ? event.key === "ArrowDown"
                    ? 0
                    : links.length - 1
                : (current + (event.key === "ArrowDown" ? 1 : -1) + links.length) % links.length;
        event.preventDefault();
        links[next].focus();
    };

    return (
        <Popover
            open={props.open}
            onOpenChange={changeOpen}
            placement={props.mobile ? "bottom-start" : "right-start"}
            gutter={8}
        >
            <Popover.Trigger
                class="flex w-full min-w-0 items-center gap-2.5 rounded-lg border-0 bg-transparent p-2 text-left text-foreground hover:bg-elevated data-[expanded]:bg-elevated"
                aria-label="Select server"
            >
                <Show
                    when={selected()}
                    fallback={
                        <span class="grid size-8 shrink-0 place-items-center rounded-lg bg-accent text-white">
                            <Icon name="discord" />
                        </span>
                    }
                >
                    {(guild) => <GuildMark guild={guild()} />}
                </Show>
                <span class="min-w-0 flex-1">
                    <span class="block truncate text-sm font-medium">
                        {selected()?.name ?? "Select a server"}
                    </span>
                    <span class="mt-0.5 block text-xs text-muted">
                        {params().guildId ? "Server" : "All servers"}
                    </span>
                </span>
                <Icon name="chevrons" size={16} />
            </Popover.Trigger>
            <Popover.Portal mount={props.portalMount}>
                <Popover.Content
                    ref={(element) => {
                        content = element;
                    }}
                    data-corvu-no-drag
                    class="panel-popover z-50 flex max-h-[min(28rem,var(--kb-popper-content-available-height))] w-72 max-w-[calc(100vw-2rem)] flex-col rounded-lg border border-line bg-surface p-1 shadow-xl outline-none"
                    onOpenAutoFocus={(event) => {
                        event.preventDefault();
                        input?.focus();
                    }}
                    onKeyDown={moveFocus}
                >
                    <Popover.Title class="sr-only">Select server</Popover.Title>
                    <div class="flex shrink-0 items-center gap-2 border-b border-line px-2 py-2 text-muted">
                        <Icon name="search" size={16} />
                        <input
                            ref={(element) => {
                                input = element;
                            }}
                            type="search"
                            aria-label="Search servers"
                            placeholder="Search servers"
                            value={search()}
                            onInput={(event) => setSearch(event.currentTarget.value)}
                            class="min-w-0 flex-1 rounded-sm border-0 bg-transparent px-1 py-1 text-sm text-foreground placeholder:text-muted"
                        />
                    </div>
                    <div class="min-h-0 overflow-y-auto overscroll-contain py-1">
                        <Show when={guilds.isPending}>
                            <p class="px-2 text-xs text-muted" role="status">
                                Loading servers...
                            </p>
                        </Show>
                        <Show when={guilds.isError}>
                            <p class="px-2 text-xs text-danger" role="alert">
                                Could not load servers.
                            </p>
                            <Button
                                size="small"
                                variant="ghost"
                                disabled={guilds.isFetching}
                                onClick={() => void guilds.refetch()}
                            >
                                Try again
                            </Button>
                        </Show>
                        <Show when={guilds.isSuccess}>
                            <nav aria-label="Servers with Yin">
                                <For
                                    each={matches()}
                                    fallback={
                                        <p class="px-2 text-xs text-muted" role="status">
                                            {installed().length
                                                ? "No matching servers."
                                                : "No servers with Yin."}
                                        </p>
                                    }
                                >
                                    {(guild) => (
                                        <Link
                                            data-guild-option
                                            to={
                                                location().pathname.endsWith("/moderation")
                                                    ? "/guilds/$guildId/moderation"
                                                    : "/guilds/$guildId"
                                            }
                                            params={{ guildId: guild.id }}
                                            class="flex items-center gap-2 rounded-md px-2 py-2 text-sm text-foreground no-underline hover:bg-elevated focus-visible:bg-elevated"
                                            aria-label={guild.name}
                                            aria-current={
                                                guild.id === params().guildId ? "page" : undefined
                                            }
                                            onClick={navigate}
                                        >
                                            <GuildMark guild={guild} class="size-6" />
                                            <span class="min-w-0 flex-1 truncate">
                                                {guild.name}
                                            </span>
                                            <Show when={guild.id === params().guildId}>
                                                <Icon name="check" size={16} />
                                            </Show>
                                        </Link>
                                    )}
                                </For>
                            </nav>
                        </Show>
                    </div>
                    <Link
                        to="/"
                        onClick={navigate}
                        class="flex shrink-0 items-center gap-2 border-t border-line px-2 py-2 text-sm text-muted no-underline hover:text-foreground"
                    >
                        <Icon name="overview" size={16} /> All servers
                    </Link>
                </Popover.Content>
            </Popover.Portal>
        </Popover>
    );
};

import { DropdownMenu } from "@kobalte/core/dropdown-menu";
import { createMutation, useQueryClient } from "@tanstack/solid-query";
import { Link, getRouteApi, useParams } from "@tanstack/solid-router";
import { Show, createEffect, createSignal, onCleanup } from "solid-js";

import { authClient } from "../lib/auth-client";
import { guildInstallUrl, managedGuildsOptions } from "../lib/guild-queries";
import { cn } from "../lib/utils";
import { Icon } from "./icon";
import { IdentityMark } from "./identity-mark";
import { GuildSelector } from "./guild-selector";
import { GuildSelectorSkeleton } from "./loading";
import { QueryBoundary } from "./query-boundary";

const dashboardRoute = getRouteApi("/_dashboard");
const navigationClass =
    "flex min-h-9 items-center gap-2.5 rounded-md px-2.5 text-sm no-underline transition-colors hover:bg-elevated hover:text-foreground";
const menuClass =
    "panel-popover z-50 min-w-60 max-w-[calc(100vw-2rem)] rounded-lg border border-line bg-surface p-1 shadow-xl outline-none";
const menuItemClass =
    "flex cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-sm text-foreground no-underline outline-none data-[highlighted]:bg-elevated data-[disabled]:opacity-50";
const triggerClass =
    "flex w-full min-w-0 items-center gap-2.5 rounded-lg border-0 bg-transparent p-2 text-left text-foreground hover:bg-elevated data-[expanded]:bg-elevated";

export const DashboardSidebar = (props: {
    mobile?: boolean;
    active?: boolean;
    menuMount?: HTMLElement;
    onNavigate?: () => void;
    onMenuOpenChange?: (open: boolean) => void;
}) => {
    const [guildMenuOpen, setGuildMenuOpen] = createSignal(false);
    const [accountMenuOpen, setAccountMenuOpen] = createSignal(false);
    createEffect(() => props.onMenuOpenChange?.(guildMenuOpen() || accountMenuOpen()));
    createEffect(() => {
        if (props.active === false) {
            setGuildMenuOpen(false);
            setAccountMenuOpen(false);
        }
    });
    onCleanup(() => props.onMenuOpenChange?.(false));
    const context = dashboardRoute.useRouteContext();
    const params = useParams({ strict: false });
    const guildId = () => params().guildId;
    const user = () => context().session.user;
    const queryClient = useQueryClient();
    const signOut = createMutation(() => ({
        mutationFn: async () => {
            const { error } = await authClient.signOut();
            if (error) throw error;
            queryClient.clear();
            window.location.assign("/login");
        },
    }));

    return (
        <div class="flex h-full min-h-0 flex-col p-2">
            <QueryBoundary
                queryKey={managedGuildsOptions().queryKey}
                fallback={<GuildSelectorSkeleton />}
            >
                <GuildSelector
                    open={guildMenuOpen()}
                    onOpenChange={setGuildMenuOpen}
                    mobile={props.mobile}
                    portalMount={props.menuMount}
                    onNavigate={props.onNavigate}
                />
            </QueryBoundary>

            <nav class="mt-5 flex-1 space-y-1 overflow-y-auto" aria-label="Dashboard navigation">
                <Link
                    to="/"
                    class={navigationClass}
                    activeOptions={{ exact: true }}
                    activeProps={{ class: "bg-elevated text-foreground", "aria-current": "page" }}
                    inactiveProps={{ class: "text-muted" }}
                    onClick={props.onNavigate}
                >
                    <Icon name="overview" size={17} /> Servers
                </Link>
                <Show when={guildId()}>
                    {(id) => (
                        <>
                            <Link
                                to="/guilds/$guildId"
                                params={{ guildId: id() }}
                                activeOptions={{ exact: true }}
                                class={navigationClass}
                                activeProps={{
                                    class: "bg-elevated text-foreground",
                                    "aria-current": "page",
                                }}
                                inactiveProps={{ class: "text-muted" }}
                                onClick={props.onNavigate}
                            >
                                <Icon name="gear" size={17} /> Server settings
                            </Link>
                            <Link
                                to="/guilds/$guildId/moderation"
                                params={{ guildId: id() }}
                                class={navigationClass}
                                activeProps={{
                                    class: "bg-elevated text-foreground",
                                    "aria-current": "page",
                                }}
                                inactiveProps={{ class: "text-muted" }}
                                onClick={props.onNavigate}
                            >
                                <Icon name="moderation" size={17} /> Moderation
                            </Link>
                        </>
                    )}
                </Show>
            </nav>

            <div class="mt-4 space-y-2">
                <a class={cn(navigationClass, "text-muted")} href={guildInstallUrl()}>
                    <Icon name="plus" size={17} /> Add a server
                </a>
                <DropdownMenu
                    modal={false}
                    open={accountMenuOpen()}
                    onOpenChange={setAccountMenuOpen}
                    placement={props.mobile ? "top-start" : "right-end"}
                    gutter={8}
                >
                    <DropdownMenu.Trigger class={triggerClass} aria-label="Account menu">
                        <IdentityMark name={user().name} image={user().image} />
                        <span class="min-w-0 flex-1">
                            <span class="block truncate text-sm font-medium">{user().name}</span>
                            <span class="mt-0.5 block truncate text-xs text-muted">
                                {user().email}
                            </span>
                        </span>
                        <Icon name="chevrons" size={16} />
                    </DropdownMenu.Trigger>
                    <DropdownMenu.Portal mount={props.menuMount}>
                        <DropdownMenu.Content data-corvu-no-drag class={menuClass}>
                            <div class="flex items-center gap-2 px-2 py-2">
                                <IdentityMark name={user().name} image={user().image} />
                                <div class="min-w-0">
                                    <p class="m-0 truncate text-sm font-medium">{user().name}</p>
                                    <p class="m-0 truncate text-xs text-muted">{user().email}</p>
                                </div>
                            </div>
                            <DropdownMenu.Separator class="my-1 border-line" />
                            <DropdownMenu.Item
                                class={menuItemClass}
                                disabled={signOut.isPending}
                                closeOnSelect={false}
                                onSelect={() => signOut.mutate()}
                            >
                                <Icon name="log-out" size={16} /> Sign out
                            </DropdownMenu.Item>
                            <Show when={signOut.isError}>
                                <p class="px-2 text-xs text-danger" role="alert">
                                    Sign out failed. Try again.
                                </p>
                            </Show>
                        </DropdownMenu.Content>
                    </DropdownMenu.Portal>
                </DropdownMenu>
            </div>
        </div>
    );
};

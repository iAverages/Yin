import { noop } from "@tanstack/solid-query";
import {
    Link,
    Outlet,
    createFileRoute,
    redirect,
    useHydrated,
    useRouterState,
} from "@tanstack/solid-router";
import { For, Show, createSignal, onMount } from "solid-js";

import { Button } from "../components/button";
import { DashboardSidebar } from "../components/dashboard-sidebar";
import { Icon } from "../components/icon";
import { MobileNavigation } from "../components/mobile-navigation";
import { PanelToaster } from "../components/panel-toaster";
import { getSession, loginRedirect } from "../lib/auth.functions";
import { managedGuildsOptions } from "../lib/guild-queries";
import { onSidebarShortcut } from "../lib/sidebar-shortcut";
import { SSR_DATA_BUDGET_MS, waitForSsrData } from "../lib/ssr-data";
import { cn } from "../lib/utils";
import type { Breadcrumb } from "../lib/breadcrumbs";

const DashboardLayout = () => {
    const breadcrumbs = useRouterState({
        select: (state) => {
            const context = state.matches.at(-1)?.context;
            return context && "breadcrumbs" in context ? context.breadcrumbs : [];
        },
    });
    const hydrated = useHydrated();
    const [sidebarOpen, setSidebarOpen] = createSignal(true);

    onMount(() => onSidebarShortcut(true, () => setSidebarOpen((open) => !open)));

    return (
        <fieldset disabled={!hydrated()} role="none" class="contents">
            <div class="flex min-h-svh bg-canvas">
                <a
                    href="#dashboard-content"
                    class="sr-only fixed top-2 left-2 z-[60] rounded-md bg-surface p-3 focus:not-sr-only"
                >
                    Skip to content
                </a>
                <aside
                    id="dashboard-sidebar"
                    aria-label="Dashboard sidebar"
                    aria-hidden={!sidebarOpen()}
                    inert={!sidebarOpen()}
                    class={cn(
                        "fixed top-0 left-0 z-20 hidden h-dvh overflow-hidden transition-[width] duration-250 ease-[cubic-bezier(0.32,0.72,0,1)] md:block",
                        sidebarOpen() ? "w-64" : "w-0",
                    )}
                >
                    <div
                        class={cn(
                            "h-full w-64 transition-transform duration-250 ease-[cubic-bezier(0.32,0.72,0,1)]",
                            sidebarOpen() ? "translate-x-0" : "-translate-x-full",
                        )}
                    >
                        <DashboardSidebar active={sidebarOpen()} />
                    </div>
                </aside>

                <div
                    id="dashboard-view"
                    class={cn(
                        "min-w-0 flex-1 bg-surface transition-[margin,border-radius,box-shadow] duration-250 ease-[cubic-bezier(0.32,0.72,0,1)]",
                        sidebarOpen() &&
                            "md:my-2 md:mr-2 md:ml-64 md:rounded-xl md:shadow-sm md:ring-1 md:ring-line/60",
                    )}
                >
                    <header class="flex h-16 items-center gap-3 px-4">
                        <Button
                            class="hidden md:inline-flex"
                            size="icon"
                            variant="ghost"
                            aria-label="Toggle sidebar"
                            aria-keyshortcuts="Control+b"
                            title="Toggle sidebar (Ctrl+B)"
                            aria-controls="dashboard-sidebar"
                            aria-expanded={sidebarOpen()}
                            onClick={() => setSidebarOpen((open) => !open)}
                        >
                            <Icon name="panel-left" size={17} />
                        </Button>
                        <MobileNavigation />
                        <span class="h-4 w-px bg-line" aria-hidden="true" />
                        <nav aria-label="Breadcrumb" class="flex items-center gap-2 text-sm">
                            <For each={breadcrumbs()}>
                                {(crumb, index) => (
                                    <>
                                        <Show when={index() > 0}>
                                            <span class="text-muted" aria-hidden="true">
                                                /
                                            </span>
                                        </Show>
                                        <Show
                                            when={index() < breadcrumbs().length - 1 && crumb.to}
                                            fallback={
                                                <span aria-current="page">{crumb.label}</span>
                                            }
                                        >
                                            {(to) => (
                                                <Link
                                                    to={to()}
                                                    class="text-muted no-underline hover:text-foreground"
                                                >
                                                    {crumb.label}
                                                </Link>
                                            )}
                                        </Show>
                                    </>
                                )}
                            </For>
                        </nav>
                    </header>
                    <main
                        id="dashboard-content"
                        tabindex="-1"
                        class="mx-auto w-full max-w-[1200px] px-4 pt-4 pb-12 outline-none md:px-8 md:pt-6"
                    >
                        <Outlet />
                    </main>
                </div>
                <PanelToaster />
            </div>
        </fieldset>
    );
};

export const Route = createFileRoute("/_dashboard")({
    context: () => ({ breadcrumbs: [] as Breadcrumb[] }),
    beforeLoad: async ({ location }) => {
        const session = await getSession();
        if (!session) throw redirect({ to: "/login", search: loginRedirect(location.href) });
        // Start one shared data budget only after authentication has completed.
        return {
            session,
            ssrDataDeadline: import.meta.env.SSR ? Date.now() + SSR_DATA_BUDGET_MS : 0,
        };
    },
    loader: async ({ context }) => {
        if (import.meta.env.SSR) {
            await waitForSsrData(
                context.queryClient.query(managedGuildsOptions()).catch(noop),
                context.ssrDataDeadline,
            );
        }
    },
    component: DashboardLayout,
});

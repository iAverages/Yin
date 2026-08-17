import Drawer from "@corvu/drawer";
import { useLocation } from "@tanstack/solid-router";
import { createEffect, createSignal, onCleanup, onMount } from "solid-js";

import { onSidebarShortcut } from "../lib/sidebar-shortcut";
import { DashboardSidebar } from "./dashboard-sidebar";
import { Icon } from "./icon";

export const MobileNavigation = () => {
    const location = useLocation();
    const [open, setOpen] = createSignal(false);
    const [trigger, setTrigger] = createSignal<HTMLButtonElement>();
    const [menuOpen, setMenuOpen] = createSignal(false);
    let readyToClose = false;
    let closeQueued = false;

    // Corvu schedules its opening transition after paint. Queue an early Escape
    // or backdrop tap so that callback cannot reopen an already-dismissed sheet.
    const changeOpen = (next: boolean) => {
        if (next) {
            readyToClose = false;
            closeQueued = false;
            setOpen(true);
        } else if (readyToClose) {
            setOpen(false);
        } else {
            closeQueued = true;
        }
    };
    const opened = () => {
        readyToClose = true;
        if (closeQueued) setOpen(false);
    };

    createEffect(() => {
        void location().pathname;
        changeOpen(false);
    });

    onMount(() => {
        const desktop = window.matchMedia("(min-width: 768px)");
        const closeOnDesktop = () => {
            if (desktop.matches) changeOpen(false);
        };
        desktop.addEventListener("change", closeOnDesktop);
        onCleanup(() => desktop.removeEventListener("change", closeOnDesktop));
        onSidebarShortcut(false, () => changeOpen(!open()));
    });

    return (
        <Drawer
            open={open()}
            onOpenChange={changeOpen}
            side="bottom"
            closeOnEscapeKeyDown={!menuOpen()}
            finalFocusEl={trigger()}
        >
            <Drawer.Trigger
                ref={setTrigger}
                class="grid size-8 place-items-center rounded-md border-0 bg-transparent text-muted hover:bg-elevated md:hidden"
                aria-label="Open navigation"
                aria-keyshortcuts="Control+b"
            >
                <Icon name="panel-left" size={17} />
            </Drawer.Trigger>
            <MobileNavigationContent
                onNavigate={() => changeOpen(false)}
                onMenuOpenChange={setMenuOpen}
                onOpened={opened}
            />
        </Drawer>
    );
};

const MobileNavigationContent = (props: {
    onNavigate: () => void;
    onMenuOpenChange: (open: boolean) => void;
    onOpened: () => void;
}) => {
    const drawer = Drawer.useContext();
    const [content, setContent] = createSignal<HTMLDivElement>();
    createEffect(() => {
        if (!drawer.isTransitioning() && drawer.openPercentage() === 1) props.onOpened();
    });

    return (
        <Drawer.Portal>
            <Drawer.Overlay
                class="fixed inset-0 z-30 data-[transitioning]:transition-colors data-[transitioning]:duration-350 data-[transitioning]:ease-[cubic-bezier(0.32,0.72,0,1)]"
                style={{
                    "background-color": `rgb(0 0 0 / ${0.6 * Math.min(1, Math.max(0, drawer.openPercentage()))})`,
                }}
            />
            <Drawer.Content
                ref={setContent}
                class="fixed inset-x-0 bottom-0 z-40 flex h-[min(32rem,85dvh)] flex-col rounded-t-2xl border-t border-line bg-canvas pb-[env(safe-area-inset-bottom)] shadow-xl outline-none after:pointer-events-none after:absolute after:inset-x-0 after:top-[calc(100%-1px)] after:h-1/2 after:bg-inherit data-[transitioning]:transition-transform data-[transitioning]:duration-350 data-[transitioning]:ease-[cubic-bezier(0.32,0.72,0,1)]"
            >
                <div
                    class="flex h-10 shrink-0 touch-none items-center justify-center"
                    data-testid="navigation-drag-handle"
                    aria-hidden="true"
                >
                    <span class="h-1 w-12 rounded-full bg-muted/40" />
                </div>
                <Drawer.Label class="sr-only">Dashboard navigation</Drawer.Label>
                <Drawer.Close
                    class="absolute top-1 right-2 grid size-9 place-items-center rounded-md bg-transparent text-muted hover:bg-elevated hover:text-foreground"
                    aria-label="Close navigation"
                    data-corvu-no-drag
                >
                    <Icon name="close" />
                </Drawer.Close>
                <div class="min-h-0 flex-1">
                    <DashboardSidebar
                        mobile
                        menuMount={content()}
                        onNavigate={props.onNavigate}
                        onMenuOpenChange={props.onMenuOpenChange}
                    />
                </div>
            </Drawer.Content>
        </Drawer.Portal>
    );
};

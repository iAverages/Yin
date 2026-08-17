import { For, Show } from "solid-js";

import { cn } from "../lib/utils";

export const Skeleton = (props: { class?: string }) => {
    return (
        <div aria-hidden="true" class={cn("animate-pulse rounded-md bg-line/70", props.class)} />
    );
};

export const GuildSelectorSkeleton = () => {
    return (
        <div class="flex items-center gap-2.5 p-2" role="status" aria-label="Loading servers">
            <Skeleton class="size-8 shrink-0" />
            <div class="flex-1 space-y-2">
                <Skeleton class="h-3.5 w-28" />
                <Skeleton class="h-3 w-16" />
            </div>
        </div>
    );
};

export const ServersSkeleton = () => {
    return (
        <div role="status" aria-label="Loading servers" aria-busy="true">
            <div class="mb-6" aria-hidden="true">
                <h1 class="m-0 text-2xl font-semibold tracking-tight">Servers</h1>
                <p class="mt-2 text-sm text-muted">Select a server to edit its settings.</p>
            </div>
            <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3" aria-hidden="true">
                <For each={Array.from({ length: 6 })}>
                    {() => (
                        <div class="flex items-center gap-3 rounded-lg border border-line bg-elevated/40 p-4">
                            <Skeleton class="size-10 shrink-0" />
                            <Skeleton class="h-4 w-3/5" />
                        </div>
                    )}
                </For>
            </div>
        </div>
    );
};

const FormSkeleton = (props: { title: string; rows?: number }) => {
    return (
        <section class="rounded-lg border border-line bg-surface p-5 md:p-6">
            <h2 class="m-0 text-base font-semibold">{props.title}</h2>
            <div class="mt-5 space-y-5">
                <For each={Array.from({ length: props.rows ?? 2 })}>
                    {() => (
                        <div class="space-y-2">
                            <Skeleton class="h-3 w-28" />
                            <Skeleton class="h-9 w-full" />
                        </div>
                    )}
                </For>
                <Skeleton class="ml-auto h-9 w-32" />
            </div>
        </section>
    );
};

const EditorSkeleton = (props: { title: string }) => {
    return (
        <section class="rounded-lg border border-line bg-surface p-5 md:p-6">
            <h2 class="m-0 text-base font-semibold">{props.title}</h2>
            <div class="mt-5 grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
                <div class="space-y-5 rounded-md border border-line bg-elevated/40 p-4">
                    <For each={[1, 2, 3]}>
                        {() => (
                            <div class="space-y-3">
                                <Skeleton class="h-3 w-24" />
                                <Skeleton class="h-9 w-full" />
                            </div>
                        )}
                    </For>
                    <Skeleton class="h-9 w-full" />
                </div>
                <div class="overflow-hidden rounded-md border border-line">
                    <div class="flex h-[42px] items-center gap-8 border-b border-line px-4">
                        <Skeleton class="h-2.5 w-16" />
                        <Skeleton class="h-2.5 w-20" />
                    </div>
                    <For each={[1, 2, 3]}>
                        {() => (
                            <div class="flex h-[58px] items-center gap-6 border-b border-line px-4 last:border-0">
                                <Skeleton class="h-3 w-2/5" />
                                <Skeleton class="h-3 w-1/4" />
                            </div>
                        )}
                    </For>
                </div>
            </div>
        </section>
    );
};

export const GuildPageSkeleton = (props: { moderation?: boolean }) => {
    return (
        <div
            role="status"
            aria-label={props.moderation ? "Loading moderation" : "Loading server settings"}
            aria-busy="true"
        >
            <div aria-hidden="true">
                <Skeleton class="h-8 w-52" />
                <p class="mt-2 mb-0 text-sm text-muted">
                    {props.moderation ? "Moderation" : "Server settings"}
                </p>
                <Show
                    when={props.moderation}
                    fallback={
                        <>
                            <div class="mt-8 grid items-start gap-4 lg:grid-cols-[minmax(0,1fr)_340px]">
                                <FormSkeleton title="General" />
                                <FormSkeleton title="Social embeds" rows={3} />
                            </div>
                            <div class="mt-4">
                                <EditorSkeleton title="Custom commands" />
                            </div>
                        </>
                    }
                >
                    <div class="mt-8">
                        <EditorSkeleton title="Punishment ladder" />
                    </div>
                </Show>
            </div>
        </div>
    );
};

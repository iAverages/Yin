import { Link } from "@tanstack/solid-router";
import { Show } from "solid-js";

import { env } from "../env";
import { Icon } from "./icon";

export const DiscordInstallResult = (props: {
    outcome: "success" | "error";
    target: "guild" | "user";
}) => {
    const succeeded = () => props.outcome === "success";
    const retryUrl = () =>
        new URL(`/install/discord/${props.target}`, env.VITE_AUTH_URL).toString();

    return (
        <main class="flex min-h-svh items-center justify-center px-5 py-12">
            <section class="w-full max-w-[400px] text-center" aria-labelledby="install-title">
                <div class="mb-6 text-muted">
                    <Icon name="discord" size={32} />
                </div>
                <h1
                    class="m-0 text-[32px] leading-tight font-semibold tracking-[-.035em]"
                    id="install-title"
                >
                    {succeeded()
                        ? "Discord authorization complete"
                        : "Discord authorization failed"}
                </h1>
                <p class="mx-auto mt-3 mb-8 max-w-[340px] leading-relaxed text-muted">
                    {succeeded()
                        ? props.target === "user"
                            ? "You can now use Yin's commands in Discord. You can close this tab."
                            : "Return to the panel to manage your server settings. You can also close this tab."
                        : "The installation was cancelled or could not be completed. Try authorizing Yin again."}
                </p>
                <div class="flex flex-col gap-3">
                    <Show when={!succeeded()}>
                        <a
                            href={retryUrl()}
                            class="inline-flex min-h-11 items-center justify-center gap-2 rounded-md bg-accent px-3.5 font-medium text-white transition-colors hover:bg-accent-hover"
                        >
                            <Icon name="discord" size={20} />
                            Try again with Discord
                        </a>
                    </Show>
                    <Link
                        to="/"
                        preload={false}
                        class="inline-flex min-h-11 items-center justify-center gap-2 rounded-md border border-line bg-elevated px-3.5 font-medium text-foreground transition-colors hover:bg-surface"
                    >
                        Go to panel
                        <Icon name="arrow-right" size={18} />
                    </Link>
                </div>
            </section>
        </main>
    );
};

import { createMutation } from "@tanstack/solid-query";
import { createFileRoute } from "@tanstack/solid-router";
import { Show } from "solid-js";

import { Button } from "../components/button";
import { Icon } from "../components/icon";
import { authClient } from "../lib/auth-client";

const Login = () => {
    const signIn = createMutation(() => ({
        mutationFn: async () => {
            const { error } = await authClient.signIn.social({
                provider: "discord",
                callbackURL: new URL("/", window.location.origin).toString(),
                scopes: ["guilds"],
            });

            if (error) {
                throw error;
            }
        },
    }));

    return (
        <main class="flex min-h-svh items-center justify-center px-5 py-12">
            <section class="w-full max-w-[360px] text-center" aria-labelledby="login-title">
                <h1
                    class="m-0 text-[32px] leading-tight font-semibold tracking-[-.035em]"
                    id="login-title"
                >
                    Sign in to Yin
                </h1>
                <p class="mx-auto mt-3 mb-8 max-w-[300px] leading-relaxed text-muted">
                    Sign in to edit your server settings.
                </p>

                <Button
                    class="min-h-11 w-full bg-[#5865f2] text-white hover:bg-[#6772f4]"
                    variant="primary"
                    disabled={signIn.isPending}
                    onClick={() => signIn.mutate()}
                >
                    <Icon name="discord" size={20} />
                    Sign in with Discord
                </Button>

                <Show when={signIn.isError}>
                    <p class="mt-3 mb-0 text-center text-xs text-[#f07982]" role="alert">
                        Discord sign-in could not be started. Try again.
                    </p>
                </Show>
            </section>
        </main>
    );
};

export const Route = createFileRoute("/login")({
    component: Login,
    head: () => ({
        meta: [
            { title: "Sign in | Yin" },
            {
                name: "description",
                content: "Sign in with Discord to manage Yin.",
            },
        ],
    }),
});

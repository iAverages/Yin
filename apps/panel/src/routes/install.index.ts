import { createFileRoute } from "@tanstack/solid-router";

import { env } from "../env";

export const Route = createFileRoute("/install/")({
    server: {
        handlers: {
            GET: () =>
                Response.redirect(
                    `https://discord.com/oauth2/authorize?client_id=${env.VITE_DISCORD_CLIENT_ID}`,
                ),
        },
    },
});

import { createFileRoute } from "@tanstack/solid-router";

import { discordInstallUrl } from "../lib/discord-install";

export const Route = createFileRoute("/install/user")({
    server: {
        handlers: {
            GET: () => Response.redirect(discordInstallUrl("user")),
        },
    },
});

import { createFileRoute } from "@tanstack/solid-router";

import { DiscordInstallResult } from "../components/discord-install-result";
import { discordInstallSearch } from "../lib/discord-install";

const InstallError = () => {
    const search = Route.useSearch();
    return <DiscordInstallResult outcome="error" target={search().target} />;
};

export const Route = createFileRoute("/install/discord/error")({
    validateSearch: discordInstallSearch,
    component: InstallError,
    head: () => ({
        meta: [
            { title: "Discord authorization failed | Yin" },
            { name: "robots", content: "noindex" },
        ],
    }),
});

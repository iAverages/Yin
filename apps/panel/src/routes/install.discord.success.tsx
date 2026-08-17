import { createFileRoute } from "@tanstack/solid-router";

import { DiscordInstallResult } from "../components/discord-install-result";
import { discordInstallSearch } from "../lib/discord-install";

const InstallSuccess = () => {
    const search = Route.useSearch();
    return <DiscordInstallResult outcome="success" target={search().target} />;
};

export const Route = createFileRoute("/install/discord/success")({
    validateSearch: discordInstallSearch,
    component: InstallSuccess,
    head: () => ({
        meta: [
            { title: "Discord authorization complete | Yin" },
            { name: "robots", content: "noindex" },
        ],
    }),
});

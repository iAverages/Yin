import { createQuery } from "@tanstack/solid-query";
import { Show, type Accessor, type JSX } from "solid-js";

import type { GuildSettings } from "../lib/api-client";
import { guildSettingsOptions, settingsQueryKey } from "../lib/guild-queries";
import { GuildPageSkeleton } from "./loading";
import { QueryBoundary } from "./query-boundary";

export const GuildSettingsContent = (props: {
    guildId: string;
    moderation?: boolean;
    children: (settings: Accessor<GuildSettings>) => JSX.Element;
}) => {
    return (
        <QueryBoundary
            queryKey={settingsQueryKey(props.guildId)}
            fallback={<GuildPageSkeleton moderation={props.moderation} />}
        >
            <GuildSettingsData {...props} />
        </QueryBoundary>
    );
};

const GuildSettingsData = (props: {
    guildId: string;
    children: (settings: Accessor<GuildSettings>) => JSX.Element;
}) => {
    const query = createQuery(() => guildSettingsOptions(props.guildId));
    return <Show when={query.data}>{props.children}</Show>;
};

import { Show } from "solid-js";

import type { ManagedGuild } from "../lib/api-client";
import { cn } from "../lib/utils";

export const IdentityMark = (props: { name: string; image?: string | null; class?: string }) => {
    return (
        <span
            class={cn(
                "grid size-8 shrink-0 place-items-center overflow-hidden rounded-lg bg-elevated text-xs font-medium text-foreground ring-1 ring-line",
                props.class,
            )}
        >
            <Show when={props.image} fallback={<span>{props.name.slice(0, 2).toUpperCase()}</span>}>
                {(src) => <img class="size-full object-cover" src={src()} alt="" />}
            </Show>
        </span>
    );
};

export const GuildMark = (props: {
    guild: Pick<ManagedGuild, "id" | "name" | "icon">;
    class?: string;
}) => {
    return (
        <IdentityMark
            name={props.guild.name}
            image={
                props.guild.icon
                    ? `https://cdn.discordapp.com/icons/${props.guild.id}/${props.guild.icon}.png?size=80`
                    : null
            }
            class={props.class}
        />
    );
};

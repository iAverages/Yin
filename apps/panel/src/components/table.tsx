import type { ComponentProps } from "solid-js";
import { splitProps } from "solid-js";

import { cn } from "../lib/utils";

export const Table = (props: ComponentProps<"table">) => {
    const [local, tableProps] = splitProps(props, ["class", "children"]);

    return (
        <div class="max-w-full overflow-x-auto">
            <table
                {...tableProps}
                class={cn(
                    "w-full min-w-[620px] border-collapse text-left md:min-w-0 [&_th]:h-[42px] [&_th]:border-b [&_th]:border-line [&_th]:px-4 [&_th]:text-[10px] [&_th]:font-semibold [&_th]:tracking-[.06em] [&_th]:text-[#7f8490] [&_th]:uppercase [&_td]:h-[58px] [&_td]:border-b [&_td]:border-line [&_td]:px-4 [&_td]:text-xs [&_td]:text-muted [&_tbody_tr:last-child_td]:border-b-0 [&_tbody_tr]:transition-colors [&_tbody_tr]:duration-[120ms] [&_tbody_tr:hover]:bg-white/[.02] [&_td:last-child]:w-[1%] [&_td:last-child]:whitespace-nowrap [&_th:last-child]:w-[1%]",
                    local.class,
                )}
            >
                {local.children}
            </table>
        </div>
    );
};

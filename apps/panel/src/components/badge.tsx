import type { ParentProps } from "solid-js";

import { cn } from "../lib/utils";

type BadgeProps = ParentProps<{
    tone?: "success" | "neutral" | "warning" | "danger";
}>;

const tones = {
    neutral: "border-line bg-elevated text-muted",
    success: "border-success/[.24] bg-success/[.09] text-[#78cda0]",
    warning: "border-warning/25 bg-warning/[.08] text-[#e8be6f]",
    danger: "border-danger/25 bg-danger/[.08] text-[#ee8991]",
};
const dots = {
    neutral: "bg-[#777d89]",
    success: "bg-success",
    warning: "bg-warning",
    danger: "bg-danger",
};

export const Badge = (props: BadgeProps) => {
    const tone = () => props.tone ?? "neutral";

    return (
        <span
            class={cn(
                "inline-flex w-max items-center gap-1.5 whitespace-nowrap rounded-full border px-2 py-1 text-[11px] font-medium",
                tones[tone()],
            )}
        >
            <span class={cn("size-1.5 rounded-full", dots[tone()])} />
            {props.children}
        </span>
    );
};

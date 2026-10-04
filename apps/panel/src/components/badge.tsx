import { cva, type VariantProps } from "class-variance-authority";
import type { ParentProps } from "solid-js";

const badgeVariants = cva(
    "inline-flex w-max items-center gap-1.5 whitespace-nowrap rounded-full border px-2 py-1 text-[11px] font-medium",
    {
        variants: {
            tone: {
                neutral: "border-line bg-elevated text-muted",
                success: "border-success/[.24] bg-success/[.09] text-[#78cda0]",
                warning: "border-warning/25 bg-warning/[.08] text-[#e8be6f]",
                danger: "border-danger/25 bg-danger/[.08] text-[#ee8991]",
            },
        },
        defaultVariants: { tone: "neutral" },
    },
);

const badgeDotVariants = cva("size-1.5 rounded-full", {
    variants: {
        tone: {
            neutral: "bg-[#777d89]",
            success: "bg-success",
            warning: "bg-warning",
            danger: "bg-danger",
        },
    },
    defaultVariants: { tone: "neutral" },
});

type BadgeProps = ParentProps<VariantProps<typeof badgeVariants>>;

export const Badge = (props: BadgeProps) => (
    <span class={badgeVariants({ tone: props.tone })}>
        <span class={badgeDotVariants({ tone: props.tone })} />
        {props.children}
    </span>
);

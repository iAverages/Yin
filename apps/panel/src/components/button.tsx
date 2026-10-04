import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "solid-js";
import { splitProps } from "solid-js";

import { cn } from "../lib/utils";

const buttonVariants = cva(
    "inline-flex cursor-pointer items-center justify-center gap-2 whitespace-nowrap rounded-md border border-transparent font-medium transition-[background-color,border-color,color,transform] duration-150 active:not-disabled:scale-[.98] disabled:cursor-not-allowed disabled:opacity-[.48]",
    {
        variants: {
            variant: {
                primary: "bg-accent text-white hover:bg-accent-hover",
                secondary:
                    "border-line bg-elevated text-foreground hover:border-[#3a3f4b] hover:bg-[#23262e]",
                ghost: "bg-transparent text-muted hover:bg-elevated hover:text-foreground",
                danger: "border-[rgb(224_90_101_/_28%)] bg-danger/[.12] text-[#ff8b94] hover:border-danger hover:bg-danger hover:text-white",
            },
            size: {
                default: "min-h-9 px-3.5",
                small: "min-h-8 px-2.5 text-xs",
                icon: "size-[34px] min-h-[34px] p-0",
            },
        },
        defaultVariants: {
            variant: "secondary",
            size: "default",
        },
    },
);

type ButtonProps = ComponentProps<"button"> & VariantProps<typeof buttonVariants>;

export const Button = (props: ButtonProps) => {
    const [local, buttonProps] = splitProps(props, ["variant", "size", "class", "children"]);

    return (
        <button
            {...buttonProps}
            class={cn(buttonVariants({ variant: local.variant, size: local.size }), local.class)}
        >
            {local.children}
        </button>
    );
};

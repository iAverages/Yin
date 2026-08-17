import type { ComponentProps, ParentProps } from "solid-js";
import { Show, splitProps } from "solid-js";

import { cn } from "../lib/utils";

export const inputClass =
    "mt-[9px] w-full rounded-md border border-line bg-elevated px-[11px] text-foreground transition-[border-color,box-shadow] duration-150 placeholder:text-[#6e7380] hover:border-[#3b404c] focus:border-accent focus:shadow-[0_0_0_3px_rgb(128_90_213_/_14%)] focus:outline-none";

export const fieldId = (props: { id?: string; label: string }) =>
    props.id ?? props.label.toLowerCase().replaceAll(" ", "-");

export const FieldLabel = (
    props: ParentProps<{ for: string; label: string; description?: string; class?: string }>,
) => {
    return (
        <label class={cn("flex min-w-0 flex-col items-stretch", props.class)} for={props.for}>
            <span class="block text-[13px] font-medium text-foreground">{props.label}</span>
            <Show when={props.description}>
                <span class="mt-1 block text-xs leading-[1.4] text-muted">{props.description}</span>
            </Show>
            {props.children}
        </label>
    );
};

type FieldProps = ComponentProps<"input"> & {
    label: string;
    description?: string;
    error?: string;
};

export const Field = (props: FieldProps) => {
    const [local, inputProps] = splitProps(props, ["label", "description", "error", "id", "class"]);
    const id = fieldId(local);

    return (
        <FieldLabel for={id} label={local.label} description={local.description}>
            <input
                {...inputProps}
                id={id}
                class={cn(
                    inputClass,
                    "h-[38px] aria-invalid:border-danger aria-invalid:focus:shadow-[0_0_0_3px_rgb(224_90_101_/_12%)]",
                    local.class,
                )}
                aria-invalid={Boolean(local.error)}
                aria-describedby={local.error ? `${id}-error` : undefined}
            />
            <Show when={local.error}>
                <span class="mt-1.5 text-xs text-[#f07982]" id={`${id}-error`}>
                    {local.error}
                </span>
            </Show>
        </FieldLabel>
    );
};

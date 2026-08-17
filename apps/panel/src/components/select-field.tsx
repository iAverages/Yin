import type { ComponentProps, JSX } from "solid-js";
import { splitProps } from "solid-js";

import { cn } from "../lib/utils";
import { FieldLabel, fieldId, inputClass } from "./field";
import { Icon } from "./icon";

type SelectFieldProps = ComponentProps<"select"> & {
    label: string;
    description?: string;
    children: JSX.Element;
};

export const SelectField = (props: SelectFieldProps) => {
    const [local, selectProps] = splitProps(props, [
        "label",
        "description",
        "children",
        "id",
        "class",
    ]);
    const id = fieldId(local);

    return (
        <FieldLabel for={id} label={local.label} description={local.description}>
            <span class="relative block">
                <select
                    {...selectProps}
                    id={id}
                    class={cn(
                        inputClass,
                        "h-[38px] cursor-pointer appearance-none pr-[34px]",
                        local.class,
                    )}
                >
                    {local.children}
                </select>
                <span class="pointer-events-none absolute top-[22px] right-[11px] text-muted">
                    <Icon name="chevron" size={14} />
                </span>
            </span>
        </FieldLabel>
    );
};

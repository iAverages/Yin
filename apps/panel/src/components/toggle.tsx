import { Show } from "solid-js";

type ToggleProps = {
    checked: boolean;
    onChange: (checked: boolean) => void;
    label: string;
    description?: string;
};

export const Toggle = (props: ToggleProps) => {
    return (
        <label class="grid min-h-[68px] cursor-pointer grid-cols-[1fr_auto] items-center gap-x-4 border-b border-line">
            <span class="min-w-0">
                <span class="block text-[13px] font-medium text-foreground">{props.label}</span>
                <Show when={props.description}>
                    <span class="mt-1 block text-xs leading-[1.4] text-muted">
                        {props.description}
                    </span>
                </Show>
            </span>
            <input
                class="peer sr-only"
                type="checkbox"
                checked={props.checked}
                onChange={(event) => props.onChange(event.currentTarget.checked)}
            />
            <span
                class="relative col-start-2 row-start-1 h-[22px] w-[38px] rounded-full bg-[#343843] transition-colors duration-150 peer-checked:bg-accent peer-checked:[&>span]:translate-x-4 peer-checked:[&>span]:bg-white peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-accent-hover"
                aria-hidden="true"
            >
                <span class="absolute top-[3px] left-[3px] size-4 rounded-full bg-[#c8cbd2] transition-[transform,background-color] duration-150" />
            </span>
        </label>
    );
};

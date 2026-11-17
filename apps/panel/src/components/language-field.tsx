import { Combobox } from "@kobalte/core/combobox";

import { type Language, languages, matchesLanguage } from "../lib/languages";
import { cn } from "../lib/utils";
import { FieldLabel, errorProps, fieldId, inputClass, invalidInputClass } from "./field";
import { Icon } from "./icon";

export const LanguageField = (props: {
    label: string;
    description?: string;
    name: string;
    value: string;
    onChange: (value: string) => void;
    onBlur: () => void;
    error?: string;
}) => {
    const id = fieldId(props);

    return (
        <FieldLabel
            for={id}
            label={props.label}
            description={props.description}
            error={props.error}
        >
            <Combobox<Language>
                options={languages}
                optionValue="code"
                optionTextValue="name"
                optionLabel="code"
                value={languages.find((language) => language.code === props.value) ?? null}
                onChange={(language) => language && props.onChange(language.code)}
                defaultFilter={matchesLanguage}
                noResetInputOnBlur
                sameWidth
                gutter={6}
                itemComponent={(item) => (
                    <Combobox.Item
                        item={item.item}
                        class="flex cursor-pointer items-center gap-2 rounded-md px-2 py-2 text-sm text-foreground outline-none data-[highlighted]:bg-elevated"
                    >
                        <Combobox.ItemLabel class="min-w-0 flex-1 truncate">
                            {item.item.rawValue.name}
                        </Combobox.ItemLabel>
                        <span class="font-mono text-xs text-muted">{item.item.rawValue.code}</span>
                        <Combobox.ItemIndicator>
                            <Icon name="check" size={16} />
                        </Combobox.ItemIndicator>
                    </Combobox.Item>
                )}
            >
                <Combobox.Control class="relative block">
                    {/* The input shows the form value, so any typed code is kept as is. */}
                    <Combobox.Input
                        id={id}
                        name={props.name}
                        class={cn(inputClass, "h-[38px] pr-[34px]", invalidInputClass)}
                        {...errorProps(id, props.error)}
                        maxlength={16}
                        placeholder="en, ja, pt-BR"
                        value={props.value}
                        onInput={(event) => props.onChange(event.currentTarget.value)}
                        onBlur={() => props.onBlur()}
                    />
                    <Combobox.Trigger
                        aria-label="Show languages"
                        class="absolute top-[9px] right-0 grid h-[38px] w-[34px] cursor-pointer place-items-center rounded-md border-0 bg-transparent text-muted hover:text-foreground"
                    >
                        <Icon name="chevron" size={14} />
                    </Combobox.Trigger>
                </Combobox.Control>
                <Combobox.Portal>
                    <Combobox.Content class="panel-popover z-50 rounded-lg border border-line bg-surface p-1 shadow-xl outline-none">
                        <Combobox.Listbox class="max-h-64 overflow-y-auto overscroll-contain outline-none" />
                    </Combobox.Content>
                </Combobox.Portal>
            </Combobox>
        </FieldLabel>
    );
};

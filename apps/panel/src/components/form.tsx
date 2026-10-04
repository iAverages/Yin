import { createFormHook, createFormHookContexts } from "@tanstack/solid-form";
import type { ComponentProps, JSX, ParentProps } from "solid-js";

import { cn } from "../lib/utils";
import { Button } from "./button";
import { Field, FieldLabel, errorProps, fieldId, inputClass, invalidInputClass } from "./field";
import { SelectField as Select } from "./select-field";
import { Toggle } from "./toggle";

const { fieldContext, formContext, useFieldContext, useFormContext } = createFormHookContexts();

type FieldOptions = { label: string; description?: string };

// Schema validators report Standard Schema issues; show the first one.
const fieldError = (errors: Array<{ message?: string } | undefined>) => errors[0]?.message;

const TextField = (
    props: FieldOptions &
        Pick<ComponentProps<"input">, "type" | "min" | "max" | "maxlength" | "placeholder">,
) => {
    const field = useFieldContext<string>();
    return (
        <Field
            {...props}
            name={field().name}
            value={field().state.value}
            onInput={(event) => field().handleChange(event.currentTarget.value)}
            onBlur={() => field().handleBlur()}
            error={fieldError(field().state.meta.errors)}
        />
    );
};

const TextareaField = (props: FieldOptions & { maxlength?: number; placeholder?: string }) => {
    const field = useFieldContext<string>();
    const id = fieldId(props);
    return (
        <FieldLabel
            for={id}
            label={props.label}
            description={props.description}
            error={fieldError(field().state.meta.errors)}
        >
            <textarea
                id={id}
                name={field().name}
                class={cn(inputClass, "min-h-32 resize-y bg-surface py-2", invalidInputClass)}
                {...errorProps(id, fieldError(field().state.meta.errors))}
                maxlength={props.maxlength}
                placeholder={props.placeholder}
                value={field().state.value}
                onInput={(event) => field().handleChange(event.currentTarget.value)}
                onBlur={() => field().handleBlur()}
            />
        </FieldLabel>
    );
};

const SelectField = (props: FieldOptions & { children: JSX.Element }) => {
    const field = useFieldContext<string>();
    return (
        <Select
            label={props.label}
            description={props.description}
            name={field().name}
            value={field().state.value}
            onChange={(event) => field().handleChange(event.currentTarget.value)}
            onBlur={() => field().handleBlur()}
        >
            {props.children}
        </Select>
    );
};

const ToggleField = (props: FieldOptions) => {
    const field = useFieldContext<boolean>();
    return (
        <Toggle
            label={props.label}
            description={props.description}
            checked={field().state.value}
            onChange={(checked) => field().handleChange(checked)}
        />
    );
};

const Form = (props: ParentProps<{ class?: string }>) => {
    const form = useFormContext();
    return (
        <form
            class={props.class}
            noValidate
            onSubmit={(event) => {
                event.preventDefault();
                void form.handleSubmit();
            }}
        >
            {props.children}
        </form>
    );
};

const SubmitButton = (props: ParentProps<{ class?: string }>) => {
    const form = useFormContext();
    const isSubmitting = form.useSelector((state) => state.isSubmitting);
    return (
        <Button type="submit" class={props.class} disabled={isSubmitting()}>
            {props.children}
        </Button>
    );
};

export const { useAppForm } = createFormHook({
    fieldContext,
    formContext,
    fieldComponents: { TextField, TextareaField, SelectField, ToggleField },
    formComponents: { Form, SubmitButton },
});

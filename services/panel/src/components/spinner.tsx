import clsx from "clsx";
import { splitProps, type Component, type JSX } from "solid-js";

export type SpinnerProps = {
    color?: string;
} & JSX.HTMLAttributes<HTMLDivElement>;

export const Spinner: Component<SpinnerProps> = (props) => {
    const [local, rest] = splitProps(props, ["class"]);

    return (
        <div class={clsx("lds-ellipsis", local.class)} {...rest}>
            <div></div>
            <div></div>
            <div></div>
            <div></div>
        </div>
    );
};

export default Spinner;

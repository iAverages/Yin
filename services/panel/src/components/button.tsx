import clsx from "clsx";
import { children, splitProps, type Component, type JSX } from "solid-js";

import Spinner from "./spinner";

type ButtonProps = {
    loading?: boolean;
} & JSX.ButtonHTMLAttributes<HTMLButtonElement>;

const Button: Component<ButtonProps> = (props) => {
    const [local, rest] = splitProps(props, ["disabled", "loading", "children", "class"]);
    const child = children(() => local.children);

    return (
        <button
            class={clsx(
                "p-4 bg-purple-700 border-0 rounded-lg text-white font-bold flex items-center justify-center hover:bg-purple-900 hover:disabled:bg-purple-700 hover:disabled:cursor-default disabled:saturate-[20%]",
                local.class
            )}
            disabled={local.disabled || local.loading}
            {...rest}
        >
            {local.loading && (
                <div
                    style={{
                        position: "absolute",
                    }}
                >
                    <Spinner color="white" />
                </div>
            )}
            <span
                style={{
                    opacity: local.loading ? 0 : 100,
                    ...(local.loading ? { transition: "opacity 0s" } : {}),
                }}
            >
                {child()}
            </span>
        </button>
    );
};

export default Button;

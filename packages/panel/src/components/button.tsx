import type { Component, JSX } from "solid-js";
import { splitProps, children } from "solid-js";

type ButtonProps = {
  loading?: boolean;
  disabled?: boolean;
  children: JSX.Element;
};

const Button: Component<ButtonProps> = (props) => {
  const [local, rest] = splitProps(props, ["disabled", "loading", "children"]);
  const child = children(() => local.children);

  return (
    <button disabled={local.disabled || local.loading} {...rest}>
      {local.loading && (
        <div
          style={{
            position: "absolute",
          }}
        >
          Loading
          {/* <Spinner color="white" /> */}
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

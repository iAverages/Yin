import React from "react";
import { FCWithChildren } from "../../types/WithChildren";

export interface ConditionalProps {
    component: React.ElementType;
    condition: boolean | (() => boolean);
}

export const ConditionalWrapper: FCWithChildren<ConditionalProps> = ({ component: Component, condition, children }) => {
    const show = typeof condition === "function" ? condition() : condition;
    const Wrapper = show ? Component : React.Fragment;
    return <Wrapper>{children}</Wrapper>;
};

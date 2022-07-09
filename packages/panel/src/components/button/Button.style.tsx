import styled from "styled-components";
import { ButtonProps } from "./Button";

export default styled.button<ButtonProps>`
    padding: 1rem;
    background-color: var(--primary);
    border: 0;
    border-radius: 1em;
    color: white;
    font-size: medium;
    font-weight: bold;
    display: flex;
    align-items: center;
    justify-content: center;
    ${(props) => (props.fullWidth ? "width: 100%" : "")};

    &:hover {
        background-color: var(--primary-dark);
        cursor: pointer;
    }

    &:hover:disabled {
        background-color: var(--primary);
        cursor: default;
    }

    &:disabled {
        filter: saturate(20%);
    }
`;

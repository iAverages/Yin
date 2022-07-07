import styled from "styled-components";

export default styled.button`
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
    transition: opacity 0s;

    &:hover {
        background-color: var(--primary-dark);
    }

    &:hover:disabled {
        background-color: var(--primary);
    }

    &:disabled {
        filter: saturate(20%);
    }
`;

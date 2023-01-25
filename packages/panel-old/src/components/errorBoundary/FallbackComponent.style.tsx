import styled from "styled-components";

export default styled.div`
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100%;
    height: 100%;
    background-color: ${(props) => props.theme.colors.error[0]};
    border: 1px solid ${(props) => props.theme.colors.error[1]};
    color: ${(props) => props.theme.font.colors.primary};
    padding: 1rem;
    border-radius: ${(props) => props.theme.sizing.borderRadius};

    & p {
        padding: 0;
        margin: 0;
    }
`;

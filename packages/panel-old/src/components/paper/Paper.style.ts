import styled from "styled-components";

export default styled.div`
    background-color: ${(props) => props.theme.colors.grey[0]};
    padding: 1.5em;
    border-radius: ${(props) => props.theme.sizing.borderRadius};
`;

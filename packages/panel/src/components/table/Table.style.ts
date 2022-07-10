import styled from "styled-components";

export default styled.table`
    width: 100%;
    table-layout: fixed;
    margin: 0;
    line-height: 2em;

    & thead th {
        color: ${(props) => props.theme.font.colors.primary};
        text-align: center;
        padding: 0 1rem;
        border-bottom: 2px solid ${(props) => props.theme.colors.grey[1]};
    }

    & tbody td {
        color: ${(props) => props.theme.font.colors.primaryDarker};
        text-align: center;
        border-bottom: 1px solid ${(props) => props.theme.colors.grey[1]};
    }

    & tbody tr:hover {
        background-color: ${(props) => props.theme.colors.grey[1]};
    }
`;

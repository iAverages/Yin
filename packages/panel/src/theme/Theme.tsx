import { ThemeProvider } from "styled-components";
import { FCWithChildren } from "../types/WithChildren";
import theme from "./yinTheme";

export interface ThemeProps {}

export const Theme: FCWithChildren<ThemeProps> = (props) => {
    return <ThemeProvider theme={theme}>{props.children}</ThemeProvider>;
};

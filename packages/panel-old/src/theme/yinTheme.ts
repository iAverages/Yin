import { DefaultTheme } from "styled-components";

export const theme: DefaultTheme = {
    font: { colors: { primary: "#d1d1d1", primaryDarker: "#b8b8b8" } },
    colors: { grey: ["#2e2e2e", "#444444", "#25262b"], red: [], error: ["#290000", "#ff8080"] },
    sizing: {
        borderRadius: "1em",
    },
};

export default theme;

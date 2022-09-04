// Used to add types to styled-components theme
import "styled-components";

declare module "styled-components" {
    export interface DefaultTheme {
        font: {
            colors: {
                primary: string;
                primaryDarker: string;
            };
        };
        colors: {
            grey: string[];
            red: string[];
            error: string[];
        };
        sizing: {
            borderRadius: string;
        };
    }
}

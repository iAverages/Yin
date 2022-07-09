import { addons } from "@storybook/addons";
import { create, themes } from "@storybook/theming";

addons.setConfig({
    theme: create({
        base: "dark",
        brandTitle: "Yin Storybook",
        appBg: "#2e2e2e",
        appContentBg: "#262626",
        barBg: "#262626",
        barSelectedColor: "white",
        colorPrimary: "#9c3eba",
        colorSecondary: "#9c3eba",
    }),
});

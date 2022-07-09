import Button, { ButtonProps } from "./Button";
import { ComponentStory, ComponentMeta } from "@storybook/react";

export default {
    component: Button,
    argTypes: { onClick: { action: "clicked" } },
} as ComponentMeta<typeof Button>;

const Template: ComponentStory<typeof Button> = (args: ButtonProps) => <Button {...args}>Default Button</Button>;

export const Primary = Template.bind({});
Primary.args = {};

export const Loading = Template.bind({});
Loading.args = { loading: true };

export const Disabled = Template.bind({});
Disabled.args = { disabled: true };

export const FullWidth = Template.bind({});
FullWidth.args = { fullWidth: true };

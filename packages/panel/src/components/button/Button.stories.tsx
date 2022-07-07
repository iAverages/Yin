import Button, { ButtonProps } from "./Button";
import { ComponentStory, ComponentMeta } from "@storybook/react";

export default {
    title: "Button",
    component: Button,
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

// export const Loading = () => <Button loading>Default Button</Button>;
// export const Disabled = () => <Button disabled>Default Button</Button>;
// export const FullWidth = () => <Button fullWidth>Default Button</Button>;

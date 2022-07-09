import Spinner, { SpinnerProps } from "./Spinner";
import { ComponentStory, ComponentMeta } from "@storybook/react";

export default {
    title: "Spinner",
    component: Spinner,
} as ComponentMeta<typeof Spinner>;

const Template: ComponentStory<typeof Spinner> = (args: SpinnerProps) => <Spinner {...args}>Default Button</Spinner>;

export const Default = Template.bind({});
Default.args = {};

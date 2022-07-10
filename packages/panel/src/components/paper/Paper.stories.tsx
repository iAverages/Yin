import Paper, { PaperProps } from "./Paper";
import { ComponentStory, ComponentMeta } from "@storybook/react";

export default {
    component: Paper,
} as ComponentMeta<typeof Paper>;

const Template: ComponentStory<typeof Paper> = (args: PaperProps) => <Paper {...args}></Paper>;

export const Default = Template.bind({});
Default.args = {};

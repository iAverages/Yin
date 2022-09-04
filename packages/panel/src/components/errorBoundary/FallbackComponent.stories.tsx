import { FallbackComponent } from "./FallbackComponent";
import { ComponentStory, ComponentMeta } from "@storybook/react";

export default {
    component: FallbackComponent,
} as ComponentMeta<typeof FallbackComponent>;

const Template: ComponentStory<typeof FallbackComponent> = (args: any) => (
    <FallbackComponent error={new Error("Storybook error!")} {...args}></FallbackComponent>
);

export const Default = Template.bind({});
Default.args = {};

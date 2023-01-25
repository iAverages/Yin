import Table, { TableProps } from "./Table";
import { ComponentStory, ComponentMeta } from "@storybook/react";

export default {
    component: Table,
} as ComponentMeta<typeof Table>;

const Template: ComponentStory<typeof Table> = (args: TableProps) => (
    <Table {...args} autoAddBody>
        <tr>
            <td>just really</td>
            <td>some</td>
            <td>rows</td>
        </tr>
        <tr>
            <td>dan</td>
            <td>is</td>
            <td>cool</td>
        </tr>
        <tr>
            <td>yin</td>
            <td>best</td>
            <td>bot</td>
        </tr>
    </Table>
);

export const Default = Template.bind({});
Default.args = { header: ["some", "table", "headers"], container: true };

export interface TableHeaderProp {
    text: string;
    span?: number;
}

export const TableHeader: React.FC<TableHeaderProp> = ({ text, span }) => (
    <th colSpan={span ?? 1}>{text.charAt(0).toUpperCase() + text.substring(1).toLowerCase()}</th>
);

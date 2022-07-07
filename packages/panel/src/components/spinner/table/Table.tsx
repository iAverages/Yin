import { FCWithChildren } from "../../../types/WithChildren";
import { isNoneEmptyArray } from "../../../utils";
import { TableHeader, TableHeaderProp } from "./TableHeader";

interface TableProps {
    header?: (string | TableHeaderProp)[] | undefined;
    autoAddBody?: boolean;
}

export const Table: FCWithChildren<TableProps> = ({ children, header, autoAddBody = true }) => (
    <table className={""}>
        {isNoneEmptyArray(header) ? (
            <thead>
                {header.map((row) => {
                    const isString = typeof row === "string";
                    return <TableHeader text={isString ? row : row.text} span={isString ? 1 : row.span} />;
                })}
            </thead>
        ) : (
            <></>
        )}
        {autoAddBody ? <tbody>{children}</tbody> : <>{children}</>}
    </table>
);

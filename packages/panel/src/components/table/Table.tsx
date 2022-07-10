import React from "react";
import { FCWithChildren } from "../../types/WithChildren";
import { isNoneEmptyArray } from "../../utils";
import Paper from "../paper/Paper";
import TableStyle from "./Table.style";
import { TableHeader, TableHeaderProp } from "./TableHeader";

export interface TableProps {
    header?: (string | TableHeaderProp)[] | undefined;
    autoAddBody?: boolean;
    container?: boolean;
}

export const Table: FCWithChildren<TableProps> = ({ children, header, autoAddBody = true, container }) => {
    const ContainerComponent = container ? Paper : React.Fragment;

    return (
        <ContainerComponent>
            <TableStyle className={""} cellSpacing="0">
                {isNoneEmptyArray(header) ? (
                    <thead>
                        <tr>
                            {header.map((row, key) => {
                                const isString = typeof row === "string";
                                const text = isString ? row : row.text;
                                const span = isString ? 1 : row.span;
                                return <TableHeader key={key} span={span} text={text} />;
                            })}
                        </tr>
                    </thead>
                ) : (
                    <></>
                )}
                {autoAddBody ? <tbody>{children}</tbody> : <>{children}</>}
            </TableStyle>
        </ContainerComponent>
    );
};

export default Table;

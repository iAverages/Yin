import { FCWithChildren } from "../../types/WithChildren";

export interface ConditionalProps {
    condition: boolean | (() => boolean);
    children: any;
}

/**
 * A conditional rendering component
 *
 * I need to work out how to do type guarding with typescript
 *
 * Right now doing something like below would result in header being possibly undefined
 *
 * \<Conditional condition={isNoneEmptyArray(header)}>
 *
 *     <thead>
 *         <tr>
 *             {header.map((row, key) => {
 *                 const isString = typeof row === "string";
 *                 const text = isString ? row : row.text;
 *                 const span = isString ? 1 : row.span;
 *                 return <TableHeader key={key} span={span} text={text} />;
 *             })}
 *         </tr>
 *     </thead>
 *
 * \</Conditional>;
 */
export const Conditional: FCWithChildren<ConditionalProps> = ({ condition, children: Children }: ConditionalProps) => {
    const show = typeof condition === "function" ? condition() : condition;
    return show ? <Children /> : null;
};

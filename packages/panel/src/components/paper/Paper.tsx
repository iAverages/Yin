import { FCWithChildren } from "../../types/WithChildren";
import PaperStyle from "./Paper.style";

export interface PaperProps {}

export const Paper: FCWithChildren<PaperProps> = ({ children, ...rest }) => {
    return <PaperStyle {...rest}>{children}</PaperStyle>;
};

export default Paper;

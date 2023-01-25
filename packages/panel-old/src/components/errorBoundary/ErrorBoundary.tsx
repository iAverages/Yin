import { ErrorBoundary as ReactErrorBoundary } from "react-error-boundary";
import { FCWithChildren } from "../../types/WithChildren";
import { FallbackComponent } from "./FallbackComponent";

export const ErrorBoundary: FCWithChildren = ({ children }) => {
    return <ReactErrorBoundary FallbackComponent={FallbackComponent}>{children}</ReactErrorBoundary>;
};

export default ErrorBoundary;

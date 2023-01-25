import { FCWithChildren } from "../../types/WithChildren";
import Spinner from "../spinner/Spinner";
import BaseButton from "./Button.style";

export interface ButtonProps {
    loading?: boolean;
    disabled?: boolean;
    fullWidth?: boolean;
}

export const Button: FCWithChildren<ButtonProps> = ({ disabled, loading, children, ...rest }) => {
    return (
        <BaseButton disabled={disabled || loading} {...rest}>
            {loading && (
                <div
                    style={{
                        position: "absolute",
                    }}
                >
                    <Spinner color="white" />
                </div>
            )}
            <span style={{ opacity: loading ? 0 : 100, ...(loading ? { transition: "opacity 0s" } : {}) }}>{children}</span>
        </BaseButton>
    );
};

export default Button;

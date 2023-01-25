import { FC } from "react";
import { FallbackProps } from "react-error-boundary";
import FallbackStyle from "./FallbackComponent.style";

export const FallbackComponent: FC<FallbackProps> = ({ error }) => {
    return (
        <FallbackStyle>
            <p>
                An error has occured: <span>{error.message}</span>
            </p>
        </FallbackStyle>
    );
};

export default FallbackComponent;

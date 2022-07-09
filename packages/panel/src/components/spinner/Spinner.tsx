import Wrapper from "./Spinner.style";

interface SpinnerProps {
    color?: string;
}

export const Spinner: React.FC<SpinnerProps> = (props) => (
    <Wrapper {...props}>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
    </Wrapper>
);

export default Spinner;

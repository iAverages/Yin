import "./App.css";
import { ErrorBoundary } from "react-error-boundary";
import { Table } from "./components/table/Table";
import useAsyncEffect from "./hooks/useAsyncEffect";
import { useState } from "react";

const WillError = () => {
    const [a, set] = useState<string[] | null>(["a", "b"]);

    setTimeout(() => {
        console.log("timed out");
        set(null);
    }, 3000);

    return (
        <h1>
            Hello{" "}
            {a.map((b) => (
                <p>{b}</p>
            ))}
        </h1>
    );
};
const Handle = ({ error }: any) => {
    return <h2>Errored: {error}</h2>;
};

function App() {
    // const [count, setCount] = useState(0);

    useAsyncEffect(async (mounted) => {
        console.log("ASync effect", mounted());
    }, []);

    return (
        <div className="App">
            <header className="App-header">
                {/* <Spinner /> */}
                <Table header={["testing", "headers", { text: "lmfao", span: 2 }]}>
                    <tr>
                        <td>a</td>
                        <td>b</td>
                        <td>c</td>
                        <td>d</td>
                    </tr>
                </Table>
                <ErrorBoundary fallback={<Handle />}>
                    <WillError />
                </ErrorBoundary>
            </header>
        </div>
    );
}

export default App;

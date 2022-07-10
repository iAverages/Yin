import "./App.css";
import Spinner from "./components/spinner/Spinner";
import { Table } from "./components/table/Table";

function App() {
    // const [count, setCount] = useState(0);

    return (
        <div className="App">
            <header className="App-header">
                <Spinner />
            </header>
            <Table header={["testing", "headers", { text: "lmfao", span: 2 }]}>
                <tr>
                    <td>a</td>
                    <td>b</td>
                    <td>c</td>
                    <td>d</td>
                </tr>
            </Table>
            {/* <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>Hello Vite + React!</p>
        <p>
          <button type="button" onClick={() => setCount((count) => count + 1)}>
            count is: {count}
          </button>
        </p>
        <p>
          Edit <code>App.tsx</code> and save to test HMR updates.
        </p>
        <p>
          <a
            className="App-link"
            href="https://reactjs.org"
            target="_blank"
            rel="noopener noreferrer"
          >
            Learn React
          </a>
          {' | '}
          <a
            className="App-link"
            href="https://vitejs.dev/guide/features.html"
            target="_blank"
            rel="noopener noreferrer"
          >
            Vite Docs
          </a>
        </p>
      </header> */}
        </div>
    );
}

export default App;

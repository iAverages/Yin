/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { Router } from "@solidjs/router";
import { client, queryClient, trpc } from "./utils/trpc";
import App from "./App";

render(
    () => (
        <trpc.Provider client={client} queryClient={queryClient}>
            <Router>
                <App />
            </Router>
        </trpc.Provider>
    ),
    document.getElementById("root") as HTMLElement
);

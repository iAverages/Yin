/* @refresh reload */
import "./index.css";

import { Router } from "@solidjs/router";
import { render } from "solid-js/web";

import App from "./App";
import { client, queryClient, trpc } from "./utils/trpc";

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

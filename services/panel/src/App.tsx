import { Route, Routes } from "@solidjs/router";
import type { Component } from "solid-js";

import Home from "./pages/home";
import Test from "./pages/test";

const App: Component = () => {
    return (
        <Routes>
            <Route path={""} component={Home} />
            <Route path={"/test"} component={Test} />
        </Routes>
    );
};

export default App;

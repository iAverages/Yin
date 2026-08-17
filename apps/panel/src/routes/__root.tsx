import type { QueryClient } from "@tanstack/solid-query";
import { SolidQueryDevtools } from "@tanstack/solid-query-devtools";
import { HeadContent, Scripts, createRootRouteWithContext } from "@tanstack/solid-router";
import "@fontsource/inter/400.css";
import "@fontsource/inter/500.css";
import "@fontsource/inter/600.css";
import "@fontsource/jetbrains-mono/400.css";

import { HydrationScript } from "solid-js/web";
import type { JSX } from "solid-js";

import styleCss from "../styles.css?url";

const RootComponent = (props: { children: JSX.Element }) => {
    return (
        <html lang="en">
            <head>
                <HydrationScript />
                <HeadContent />
            </head>
            <body>
                {props.children}
                <SolidQueryDevtools buttonPosition="bottom-right" />
                <Scripts />
            </body>
        </html>
    );
};

export const Route = createRootRouteWithContext<{ queryClient: QueryClient }>()({
    head: () => ({
        meta: [
            { charSet: "utf-8" },
            { name: "viewport", content: "width=device-width, initial-scale=1" },
            { title: "Yin control panel" },
        ],
        links: [{ rel: "stylesheet", href: styleCss }],
    }),
    shellComponent: RootComponent,
});

import { createRouter as createTanStackRouter } from "@tanstack/solid-router";
import { setupRouterSsrQueryIntegration } from "@tanstack/solid-router-ssr-query";
import { routeTree } from "./routeTree.gen";

import { ApiRequestError } from "./lib/api-error";

import { getContext } from "./integrations/tanstack-query/provider";

export const getRouter = () => {
    const context = getContext();
    const router = createTanStackRouter({
        routeTree,

        context,

        scrollRestoration: true,
        defaultPreload: "intent",
        defaultPreloadStaleTime: 0,
        defaultPendingMs: 150,
        defaultPendingMinMs: 150,
    });

    setupRouterSsrQueryIntegration({
        router,
        queryClient: context.queryClient,
        dehydrateOptions: { shouldRedactErrors: (error) => !(error instanceof ApiRequestError) },
    });

    return router;
};

declare module "@tanstack/solid-router" {
    interface Register {
        router: ReturnType<typeof getRouter>;
    }
}

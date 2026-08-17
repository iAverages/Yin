import { QueryClient, environmentManager } from "@tanstack/solid-query";

import { isRetryableError } from "../../lib/api-error";

export const getContext = () => {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                retry: (failureCount, error) =>
                    !environmentManager.isServer() && failureCount < 3 && isRetryableError(error),
                throwOnError: (_error, query) => query.state.data === undefined,
            },
        },
    });
    return {
        queryClient,
    };
};

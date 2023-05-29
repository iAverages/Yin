import { QueryClient } from "@tanstack/solid-query";
import { httpBatchLink } from "@trpc/client";
import { createTRPCSolid } from "solid-trpc";
import superjson from "superjson";

import { type AppRouterType } from "@yin/trpc";

export const trpc = createTRPCSolid<AppRouterType>();
export const client = trpc.createClient({
    transformer: superjson,
    links: [
        httpBatchLink({
            url: import.meta.env.DEV ? "http://localhost:3001/api/trpc" : "/api/trpc",
        }),
    ],
});
export const queryClient = new QueryClient();

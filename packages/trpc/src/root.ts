import { devRouter } from "./router/dev";
import { createTRPCRouter } from "./trpc";

export const appRouter = createTRPCRouter({
    dev: devRouter,
});

// export type definition of API
export type AppRouterType = typeof appRouter;

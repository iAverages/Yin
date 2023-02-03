import { createTRPCRouter, publicProcedure } from "../trpc";

export const devRouter = createTRPCRouter({
    getDevMessage: publicProcedure.query(({}) => {
        return "ctx.session";
    }),
});

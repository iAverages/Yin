import { initTRPC } from "@trpc/server";
import superjson from "superjson";
import { Context } from "context";

export const t = initTRPC.context<Context>().create({
    transformer: superjson,
    errorFormatter({ shape }) {
        return shape;
    },
});

export const createTRPCRouter = t.router;

export const publicProcedure = t.procedure;

/**
 * Reusable middleware that enforces users are logged in before running the
 * procedure
 */
const enforceUserIsAuthed = t.middleware(({ ctx: _ctx, next }) => {
    //   if (!ctx.session?.user) {
    //     throw new TRPCError({ code: "UNAUTHORIZED" });
    //   }
    return next({
        ctx: {
            // infers the `session` as non-nullable
            // session: { ...ctx.session, user: ctx.session.user },
        },
    });
});

export const protectedProcedure = t.procedure.use(enforceUserIsAuthed);

import { type inferAsyncReturnType } from "@trpc/server";
import { type CreateFastifyContextOptions } from "@trpc/server/adapters/fastify";

import { prisma } from "@yin/db";

export function createContext(_opts: CreateFastifyContextOptions) {
    // Will add session in the future
    return { prisma, session: {} };
}

export type Context = inferAsyncReturnType<typeof createContext>;

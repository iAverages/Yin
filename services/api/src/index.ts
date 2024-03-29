import cors from "@fastify/cors";
import { fastifyTRPCPlugin } from "@trpc/server/adapters/fastify";
import fastify from "fastify";
import sourceMapSupport from "source-map-support";

import { prisma } from "@yin/db";
import { appRouter, createContext } from "@yin/trpc";

sourceMapSupport.install();

const server = fastify({
    maxParamLength: 5000,
});

server.register(fastifyTRPCPlugin, {
    prefix: "/api/trpc",
    trpcOptions: { router: appRouter, createContext },
});

server.register(cors, {
    origin: "*",
});

server.get("/get", async () => {
    await prisma.guild.create({
        data: {
            id: "123",
            owner: "dan2",
        },
    });
    return { hello: "world" };
});

(async () => {
    try {
        await server.listen({ port: 3001 });
    } catch (err) {
        server.log.error(err);
        process.exit(1);
    }
})();

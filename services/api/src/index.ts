import { fastifyTRPCPlugin } from "@trpc/server/adapters/fastify";
import fastify from "fastify";
import { appRouter, createContext } from "@yin/trpc";
import { prisma } from "@yin/db";

const server = fastify({
    maxParamLength: 5000,
});

server.register(fastifyTRPCPlugin, {
    prefix: "/trpc",
    trpcOptions: { router: appRouter, createContext },
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

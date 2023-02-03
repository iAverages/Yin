import { fastifyTRPCPlugin } from "@trpc/server/adapters/fastify";
import fastify from "fastify";
import { appRouter, createContext } from "@yin/trpc";

const server = fastify({
    maxParamLength: 5000,
});

server.register(fastifyTRPCPlugin, {
    prefix: "/trpc",
    trpcOptions: { router: appRouter, createContext },
});

(async () => {
    try {
        await server.listen({ port: 3000 });
    } catch (err) {
        server.log.error(err);
        process.exit(1);
    }
})();

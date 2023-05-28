import grpc from "@grpc/grpc-js";

import { logger } from "@yin/common";
import { prisma } from "@yin/db";
import { database } from "@yin/grpc";

import { env } from "~/env";
import { createGuildService } from "~/guilds";
import { createInternalService } from "~/internal";
import { createUserService } from "~/user";

export type InternalServiceProps = {
    prisma: typeof prisma;
    logger: typeof logger;
};

export type InternalService<K extends keyof database.DatabaseServer> = (
    props: InternalServiceProps
) => Pick<database.DatabaseServer, K>;

export const startGrpcServer = async () => {
    const server = new grpc.Server();

    const guildService = createGuildService({ prisma, logger });
    const internalService = createInternalService({ prisma, logger });
    const userService = createUserService({ prisma, logger });

    const databaseServiceImp: database.DatabaseServer = {
        ...guildService,
        ...internalService,
        ...userService,
    };

    server.addService(database.DatabaseService, databaseServiceImp);

    server.bindAsync(env.YIN_DATABASE_GRPC_HOST, grpc.ServerCredentials.createInsecure(), (err, port) => {
        if (err) {
            logger.error(err);
            return;
        }
        logger.info(`gRPC server listening on ${env.YIN_DATABASE_GRPC_HOST} ${port}`);
        server.start();
    });
};

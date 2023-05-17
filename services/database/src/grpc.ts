import grpc from "@grpc/grpc-js";

import { logger } from "@yin/common";
import { prisma } from "@yin/db";
import { database } from "@yin/grpc";

import { env } from "~/env";

const { DatabaseService } = database;

export const startGrpcServer = async () => {
    const server = new grpc.Server();

    const DatabaseServiceImp: database.DatabaseServer = {
        addGuild: async (call, callback) => {
            try {
                const data = await prisma.guild.create({
                    data: {
                        icon: call.request.icon,
                        id: call.request.id,
                        name: call.request.name,
                    },
                });

                console.log(data);
                callback(null, { success: true });
            } catch (err) {
                logger.error(err);
                callback({ message: "Failed to add guild", name: "RECORD_CREATE_FAILURE" }, { success: false });
            }
        },
        getGuild: (call, callback) => {
            console.log(call.request);
            callback(null, { icon: "", name: "", id: "" });
        },
        logEvent: async ({ request: event }, callback) => {
            try {
                const data = await prisma.eventLog.create({
                    data: {
                        createdAt: new Date(),
                        discordEvent: {
                            connect: {
                                name: event.discordEvent,
                            },
                        },
                        guild: {
                            connect: {
                                id: event.discordGuildId,
                            },
                        },
                        user: {
                            connect: {
                                id: event.discordUserId,
                            },
                        },
                    },
                });

                console.log(data);
                callback(null, { success: true });
            } catch (err) {
                logger.error(err);
                callback({ message: "Failed to update event log", name: "RECORD_CREATE_FAILURE" }, { success: false });
            }
        },
        removeGuild: (call, callback) => {
            console.log(call.request);
            callback(null, { success: true });
        },
    };

    server.addService(DatabaseService, DatabaseServiceImp);

    server.bindAsync(
        `localhost:${env.YIN_DATABASE_GRPC_PORT}`,
        grpc.ServerCredentials.createInsecure(),
        (err, port) => {
            if (err) {
                logger.error(err);
                return;
            }
            logger.info(`Listening on ${port}`);
            server.start();
        }
    );
};

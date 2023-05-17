import grpc from "@grpc/grpc-js";

import { database } from "@yin/grpc";

import { env } from "~/env";

const { DatabaseService } = database;

export const start = () => {
    const server = new grpc.Server();

    const DatabaseServiceImp: database.DatabaseServer = {
        addGuild: (call, callback) => {
            console.log(call.request);
            callback(null, { success: true });
        },
        getGuild: (call, callback) => {
            console.log(call.request);
            callback(null, { icon: "", name: "", id: "" });
        },
        logEvent: (call, callback) => {
            console.log(call.request);
            callback(null, { success: true });
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
                console.log(err);
                return;
            }
            console.log(`Listening on ${port}`);
            server.start();
        }
    );
};

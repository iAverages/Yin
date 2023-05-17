import grpc from "@grpc/grpc-js";

import { database as databaseService, worker as workerService } from "@yin/grpc";

import { env } from "./env";

export const createWorkerConnection = () => {
    const worker = new workerService.WorkerClient(
        `localhost:${env.YIN_WORKER_GRPC_PORT}`,
        grpc.credentials.createInsecure()
    );
    return worker;
};

export const createDatabaseConnection = () => {
    const worker = new databaseService.DatabaseClient(
        `localhost:${env.YIN_DATABASE_GRPC_PORT}`,
        grpc.credentials.createInsecure()
    );
    return worker;
};

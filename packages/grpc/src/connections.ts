import grpc from "@grpc/grpc-js";

import { __env } from "@yin/common";

import { database as databaseService, worker as workerService } from "./index";

export const createWorkerConnection = () => {
    const worker = new workerService.WorkerClient(
        `localhost:${__env.YIN_WORKER_GRPC_PORT}`,
        grpc.credentials.createInsecure()
    );

    return worker;
};

export const createDatabaseConnection = () => {
    const worker = new databaseService.DatabaseClient(
        `localhost:${__env.YIN_DATABASE_GRPC_PORT}`,
        grpc.credentials.createInsecure()
    );

    return worker;
};

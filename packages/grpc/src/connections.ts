import grpc from "@grpc/grpc-js";

import { __env } from "@yin/common";

import { database as databaseService, worker as workerService } from "./index";

export const createWorkerConnection = () => {
    const worker = new workerService.WorkerClient(__env.YIN_WORKER_GRPC_HOST, grpc.credentials.createInsecure());

    return worker;
};

export const createDatabaseConnection = () => {
    const worker = new databaseService.DatabaseClient(__env.YIN_DATABASE_GRPC_HOST, grpc.credentials.createInsecure());

    return worker;
};

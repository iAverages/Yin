import grpc from "@grpc/grpc-js";

import { worker as workerService } from "@yin/grpc";

import { env } from "./env";

const worker = new workerService.WorkerClient(
    `localhost:${env.YIN_WORKER_GRPC_PORT}`,
    grpc.credentials.createInsecure()
);

export default worker;

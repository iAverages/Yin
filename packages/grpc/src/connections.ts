import grpc, { InterceptingCall } from "@grpc/grpc-js";

import { __env, metrics } from "@yin/common";

import { database as databaseService, worker as workerService } from "./index";

const grpcPacketTimings = new metrics.Histogram({
    help: "Time taken to send a packet",
    name: "grpc_packet_timings",
    labelNames: ["endpoint"] as const,
});

export const createWorkerConnection = () => {
    const worker = new workerService.WorkerClient(__env.YIN_WORKER_GRPC_HOST, grpc.credentials.createInsecure(), {
        interceptors: [
            (options, next) => {
                return new InterceptingCall(next(options), {
                    start: (metadata, _listener, next) => {
                        const timer = grpcPacketTimings.startTimer();
                        return next(metadata, {
                            onReceiveMessage: (message, next) => {
                                timer({ endpoint: options.method_definition.path });
                                return next(message);
                            },
                        });
                    },
                });
            },
        ],
    });

    return worker;
};

export const createDatabaseConnection = () => {
    const worker = new databaseService.DatabaseClient(__env.YIN_DATABASE_GRPC_HOST, grpc.credentials.createInsecure());

    return worker;
};

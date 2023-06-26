import grpc, { InterceptingCall } from "@grpc/grpc-js";

import { __env, metrics } from "@yin/common";

import { database as databaseService, worker as workerService } from "./index";

const grpcPacketTimings = new metrics.Histogram({
    help: "Time taken to send a packet",
    name: "grpc_request_duration_seconds",
    labelNames: ["endpoint", "service", "host"] as const,
});

const createGrpcRequestDurationInterceptor = ({
    service,
    options,
    next,
}: {
    service: string;
    options: grpc.InterceptorOptions;
    next: grpc.NextCall;
}) => {
    return new InterceptingCall(next(options), {
        start: (metadata, _listener, next) => {
            const timer = grpcPacketTimings.startTimer();
            return next(metadata, {
                onReceiveMessage: (message, next) => {
                    timer({
                        endpoint: options.method_definition.path,
                        service,
                        host: options.host,
                    });
                    return next(message);
                },
            });
        },
    });
};

export const createWorkerConnection = () => {
    const worker = new workerService.WorkerClient(__env.YIN_WORKER_GRPC_HOST, grpc.credentials.createInsecure(), {
        interceptors: [(options, next) => createGrpcRequestDurationInterceptor({ service: "worker", options, next })],
    });

    return worker;
};

export const createDatabaseConnection = () => {
    const database = new databaseService.DatabaseClient(
        __env.YIN_DATABASE_GRPC_HOST,
        grpc.credentials.createInsecure(),
        {
            interceptors: [
                (options, next) => createGrpcRequestDurationInterceptor({ service: "database", options, next }),
            ],
        }
    );

    return database;
};

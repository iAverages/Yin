import { z } from "zod";

import { DefaultEnv, DefaultServiceMeta, setup, validateEnvVars } from "@yin/common";
import { type database as grpcDatabase, type worker as grpcWorker } from "@yin/grpc";

import * as grpc from "~/grpc";

const env = validateEnvVars(z.object({}));

export const createService = () => {
    return setup<GatewayServiceMeta>("gateway", {
        services: {
            worker: grpc.createWorkerConnection(),
            database: grpc.createDatabaseConnection(),
        },
        env,
    });
};

type GatewayServiceMeta = {
    services: {
        worker: grpcWorker.WorkerClient;
        database: grpcDatabase.DatabaseClient;
    };
    env: DefaultEnv;
};

export type ServiceMeta = DefaultServiceMeta & GatewayServiceMeta;

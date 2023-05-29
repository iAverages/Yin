import { z } from "zod";

import { type DefaultEnv, type DefaultServiceMeta, setup, validateEnvVars } from "@yin/common";
import { connections, type database as grpcDatabase, type worker as grpcWorker } from "@yin/grpc";

const env = validateEnvVars(z.object({}));

export const createService = () => {
    return setup<GatewayServiceMeta>("gateway", {
        services: {
            worker: connections.createWorkerConnection(),
            database: connections.createDatabaseConnection(),
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

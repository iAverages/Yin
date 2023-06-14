import { z } from "zod";

import { setup, validateEnvVars, type DefaultEnv, type DefaultServiceMeta } from "@yin/common";
import { connections, type database as grpcDatabase } from "@yin/grpc";

const env = validateEnvVars(z.object({}));

export const createService = async () => {
    return await setup<WorkerServiceMeta>("worker", {
        services: {
            database: connections.createDatabaseConnection(),
        },
        env,
    });
};

type WorkerServiceMeta = {
    services: {
        database: grpcDatabase.DatabaseClient;
    };
    env: DefaultEnv;
};

export type ServiceMeta = DefaultServiceMeta & WorkerServiceMeta;

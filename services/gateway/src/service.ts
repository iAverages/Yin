import * as Sentry from "@sentry/node";
import { z } from "zod";

import { setup, validateEnvVars, type DefaultEnv, type DefaultServiceMeta } from "@yin/common";
import api from "@yin/discord";
import { connections, type database as grpcDatabase, type worker as grpcWorker } from "@yin/grpc";

const env = validateEnvVars(z.object({}));

Sentry.init({
    dsn: env.SENTRY_DSN,
    tracesSampleRate: 1.0,
});

export const createService = async () => {
    const botUser = await api.user.getCurrentUser();
    if (!botUser.success) {
        throw new Error("Bot user not found");
    }

    return setup<GatewayServiceMeta>("gateway", {
        services: {
            worker: connections.createWorkerConnection(),
            database: connections.createDatabaseConnection(),
        },
        env,
        botUser: botUser.data,
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

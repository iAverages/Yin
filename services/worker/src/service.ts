import { z } from "zod";

import { setup, validateEnvVars, type DefaultEnv, type DefaultServiceMeta } from "@yin/common";
import api from "@yin/discord";
import { connections, type database as grpcDatabase } from "@yin/grpc";

const env = validateEnvVars(z.object({}));

export const createService = async () => {
    const botUser = await api.user.getCurrentUser();
    if (!botUser.success) {
        throw new Error("Bot user not found");
    }

    return setup<WorkerServiceMeta>("gateway", {
        services: {
            database: connections.createDatabaseConnection(),
        },
        env,
        botUser: botUser.data,
    });
};

type WorkerServiceMeta = {
    services: {
        database: grpcDatabase.DatabaseClient;
    };
    env: DefaultEnv;
};

export type ServiceMeta = DefaultServiceMeta & WorkerServiceMeta;

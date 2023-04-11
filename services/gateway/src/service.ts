import { z } from "zod";

import { DefaultEnv, DefaultServiceMeta, setup, validateEnvVars } from "@yin/common";
import { type worker as grpcWorker } from "@yin/grpc";

import * as grpc from "~/grpc";

const env = validateEnvVars(z.object({}));
const worker = grpc.createWorkerConnection();

export const createService = () => {
    return setup("gateway", {
        services: {
            worker,
        },
        env,
    });
};

type GatewayServiceMeta = {
    services: {
        worker: grpcWorker.WorkerClient;
    };
    env: DefaultEnv;
};

export type ServiceMeta = DefaultServiceMeta & GatewayServiceMeta;

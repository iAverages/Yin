import grpc from "@grpc/grpc-js";

import { logger } from "@yin/common";
import { worker } from "@yin/grpc";

import { env } from "~/env";
import { createService } from "~/service";
import { createMessageService } from "~/services/message";

export type InternalServiceProps = {
    logger: typeof logger;
    service: Awaited<ReturnType<typeof createService>>;
};

export type InternalService<K extends keyof worker.WorkerServer> = (
    props: InternalServiceProps
) => Pick<worker.WorkerServer, K>;

export const startGrpcServer = async () => {
    const server = new grpc.Server();
    const service = await createService();

    const mesageService = createMessageService({ service, logger });

    const workerServiceImp: worker.WorkerServer = {
        ...mesageService,
    };

    server.addService(worker.WorkerService, workerServiceImp);

    server.bindAsync(env.YIN_WORKER_GRPC_HOST, grpc.ServerCredentials.createInsecure(), (err, port) => {
        if (err) {
            logger.error(err);
            return;
        }
        logger.info(`gRPC server listening on ${env.YIN_WORKER_GRPC_HOST} ${port}`);
        server.start();
    });
};

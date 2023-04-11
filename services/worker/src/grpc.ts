import grpc from "@grpc/grpc-js";
import { worker } from "@yin/grpc";

import { env } from "./env";

const { WorkerService, StatusReply } = worker;
export const start = () => {
    const server = new grpc.Server();

    const WorkerServiceImp: worker.WorkerServer = {
        handlePacket: (call, callback) => {
            console.log(call.request);
            callback(null, StatusReply.create({ success: false, message: call.request.body }));
        },
    };

    server.addService(WorkerService, WorkerServiceImp);

    server.bindAsync(`localhost:${env.YIN_WORKER_GRPC_PORT}`, grpc.ServerCredentials.createInsecure(), (err, port) => {
        if (err) {
            console.log(err);
            return;
        }
        console.log(`Listening on ${port}`);
        server.start();
    });
};

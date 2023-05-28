import grpc from "@grpc/grpc-js";

import api from "@yin/discord";
import { worker } from "@yin/grpc";

import { env } from "./env";

const { WorkerService, StatusReply } = worker;
export const start = () => {
    const server = new grpc.Server();

    const WorkerServiceImp: worker.WorkerServer = {
        handlePacket: async (call, callback) => {
            console.log(call.request);
            const packet = JSON.parse(call.request.body) as any;
            const res = await api.interaction.respond(
                {
                    type: 4,
                    data: {
                        content: `Hello world! - ${env.K3S_POD_NAME}`,
                    },
                },
                {
                    "interaction.id": packet.d.id,
                    "interaction.token": packet.d.token,
                }
            );
            console.log(res);
            callback(null, StatusReply.create({ success: false, message: call.request.body }));
        },
    };

    server.addService(WorkerService, WorkerServiceImp);

    server.bindAsync(env.YIN_WORKER_GRPC_HOST, grpc.ServerCredentials.createInsecure(), (err, port) => {
        if (err) {
            console.log(err);
            return;
        }
        console.log(`Listening on ${port}`);
        server.start();
    });
};

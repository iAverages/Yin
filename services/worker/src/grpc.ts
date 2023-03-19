import { WorkerServer, WorkerService, StatusReply } from "@yin/grpc";
import grpc from "@grpc/grpc-js";

export const start = () => {
    const server = new grpc.Server();

    const WorkerServiceImp: WorkerServer = {
        handlePacket: (call, callback) => {
            console.log(call.request);
            callback(null, StatusReply.create({ success: false, message: call.request.body }));
        },
    };

    server.addService(WorkerService, WorkerServiceImp);

    server.bindAsync(`localhost:50051`, grpc.ServerCredentials.createInsecure(), (err, port) => {
        if (err) {
            console.log(err);
            return;
        }
        console.log(`Listening on ${port}`);
        server.start();
    });
};

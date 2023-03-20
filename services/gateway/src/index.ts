import sourceMapSupport from "source-map-support";
sourceMapSupport.install();

// import { env } from "./envVars";

import { WebSocket } from "./WebSocket";

const socket = new WebSocket();
socket.connect();

import grpc from "./grpc";
import { worker } from "@yin/grpc";

// console.log(env);
setInterval(() => {
    grpc.handlePacket(worker.Packet.create({ body: "hello this is a test" }), (err, lol) => {
        console.log(err);
        console.log(lol);
    });
}, 5000);

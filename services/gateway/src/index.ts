// import { env } from "./envVars";

import { worker } from "@yin/grpc";
import sourceMapSupport from "source-map-support";
import { WebSocket } from "~/WebSocket";
import grpc from "~/grpc";

sourceMapSupport.install();

const socket = new WebSocket();
socket.connect();

// import grpc from "./grpc";
// import { worker } from "@yin/grpc";

// // console.log(env);
// setInterval(() => {
//     grpc.handlePacket(worker.Packet.create({ body: "hello this is a test" }), (err, lol) => {
//         console.log(err);
//         console.log(lol);
//     });
// }, 5000);

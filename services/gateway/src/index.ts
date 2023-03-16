import sourceMapSupport from "source-map-support";
sourceMapSupport.install();

import("./env");

import { WebSocket } from "./WebSocket";

const socket = new WebSocket();
socket.connect();

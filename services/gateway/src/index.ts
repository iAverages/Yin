import sourceMapSupport from "source-map-support";
sourceMapSupport.install();

import { WebSocket } from "./WebSocket";
import { validateEnvVars } from "@yin/common";

validateEnvVars();

const socket = new WebSocket();
socket.connect();

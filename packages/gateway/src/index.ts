// Makes logs use ts mappings
import "source-map-support/register";
import { WebSocket } from "./WebSocket";
import { validateEnvVars } from "@yin/common";

validateEnvVars();

const socket = new WebSocket();
socket.connect();
//

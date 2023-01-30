// Makes logs use ts mappings
import "source-map-support/register";
import { WebSocket } from "./WebSocket";
// import { validateEnvVars } from "@yin/common";
// import { z } from "zod";

// validateEnvVars(
//     z.object({
//         TEST: z.string(),
//     })
// );

const socket = new WebSocket();
socket.connect();

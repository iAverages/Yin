import "@yin/common/src/sourceMap";

import { createService } from "~/service";
import { WebSocket } from "~/WebSocket";

const service = createService();

const socket = new WebSocket(service);
socket.connect();

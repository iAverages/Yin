import _sourceMap from "@yin/common/src/sourceMap";

import { WebSocket } from "~/WebSocket";
import { createService } from "~/service";

const service = createService();

const socket = new WebSocket(service);
socket.connect();

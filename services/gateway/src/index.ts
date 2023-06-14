import "@yin/common/src/sourceMap";

import { createService } from "~/service";
import { WebSocket } from "~/WebSocket";

(async () => {
    const service = await createService();

    const socket = new WebSocket(service);
    socket.connect();
})();

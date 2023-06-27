import { createPrometheusServer, logger } from "@yin/common";

import "@yin/common/src/sourceMap";

import { env } from "~/env";
import { createService } from "~/service";
import { WebSocket } from "~/WebSocket";

(async () => {
    const prometheus = createPrometheusServer();
    const service = await createService();
    logger.info("Starting gateway service");

    const socket = new WebSocket(service);
    socket.connect();
    prometheus.listen(env.YIN_PROMETHEUS_PORT);
})();

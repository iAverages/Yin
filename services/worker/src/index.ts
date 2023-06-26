import sourceMapSupport from "source-map-support";

import { createPrometheusServer, logger } from "@yin/common";

import { startGrpcServer } from "./grpc";

sourceMapSupport.install();

(async () => {
    const prometheus = createPrometheusServer();
    startGrpcServer().catch((err) => {
        logger.error(err);
        process.exit(1);
    });
    prometheus.listen(8080);
})();

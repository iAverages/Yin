import sourceMapSupport from "source-map-support";

import { logger } from "@yin/common";

import { env } from "./env";
import { startGrpcServer } from "./grpc";

sourceMapSupport.install();

console.log(env);
startGrpcServer().catch((err) => {
    logger.error(err);
    process.exit(1);
});

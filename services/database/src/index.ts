import _sourceMap from "@yin/common/src/sourceMap";

import { prepareDatabase } from "~/database";
import { startGrpcServer } from "./grpc";

const start = async () => {
    await prepareDatabase();
    startGrpcServer();
};

start();

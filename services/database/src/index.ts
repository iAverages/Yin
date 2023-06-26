import "@yin/common/src/sourceMap";

import { createPrometheusServer } from "@yin/common";
import { prisma } from "@yin/db";

import { prepareDatabase } from "~/database";
import { startGrpcServer } from "./grpc";

const start = async () => {
    await prepareDatabase();
    startGrpcServer();

    const prometheus = createPrometheusServer({
        additional: [await prisma.$metrics.prometheus()],
    });
    prometheus.listen(8080);
};

start();

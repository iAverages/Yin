import _sourceMap from "@yin/common/src/sourceMap";

// import sourceMapSupport from "source-map-support";

import { env } from "./env";
import { start } from "./grpc";

// sourceMapSupport.install();

console.log(env);
start();

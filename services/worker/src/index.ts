import sourceMapSupport from "source-map-support";
sourceMapSupport.install();

import { start } from "./grpc";

import { env } from "./env";

console.log(env);
start();

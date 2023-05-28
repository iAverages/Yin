import path from "path";
import dotenv from "dotenv";
// Always load dotenv when common is imported
dotenv.config({ path: path.resolve("../../.env") });

export { consts } from "./consts";
export { setup, type DefaultServiceMeta } from "./setup";

export { logger } from "./logger"; 

// __env export is used for other packages, services should not use this export directly
// instead, use validateEnvVars
export { env as __env, globalSchema, validateEnvVars, type DefaultEnv } from "./env";

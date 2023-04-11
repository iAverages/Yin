import path from "path";
import dotenv from "dotenv";
// Always load dotenv when common is imported
dotenv.config({ path: path.resolve("../../.env") });

import { consts } from "./consts";
import { env as __env, globalSchema, validateEnvVars } from "./env";

// __env export is used for other packages, services should not use this export directly
// instead, use validateEnvVars
export { consts, validateEnvVars, globalSchema, __env };

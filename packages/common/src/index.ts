// Once build logs will show TS file paths
import path from "path";

// Always load dotenv when common is imported
import dotenv from "dotenv";
dotenv.config({ path: path.resolve("../../.env") });

import { consts } from "./consts";
import { globalSchema, validateEnvVars, env as __env } from "./env/index";

// __env export is used for other packages, services should not use this export directly
// instead, use validateEnvVars
export { consts, validateEnvVars, globalSchema, __env };

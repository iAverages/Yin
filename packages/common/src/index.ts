// Once build logs will show TS file paths
import path from "path";

// Always load dotenv when common is imported
import dotenv from "dotenv";
dotenv.config({ path: path.resolve("../../.env") });

import { consts } from "./consts";
import { globalSchema, validateEnvVars, envWithPrefix } from "./env/index";

export { consts, validateEnvVars, globalSchema, envWithPrefix };

import path from "path";
// Always load dotenv when common is imported
import dotenv from "dotenv";

import { consts } from "./consts";
import { globalSchema, validateEnvVars } from "./env/index";

dotenv.config({ path: path.resolve("../../.env") });

export { consts, validateEnvVars, globalSchema };

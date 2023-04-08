import path from "path";
import dotenv from "dotenv";
dotenv.config({ path: path.resolve("../../.env") });

// Always load dotenv when common is imported

import { globalSchema, validateEnvVars } from "~/env";



export { validateEnvVars, globalSchema };

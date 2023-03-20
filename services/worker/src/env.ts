import { validateEnvVars } from "@yin/common";
import { z } from "zod";

const schema = z.object({});

const env = validateEnvVars(schema);

export { env };

import { z } from "zod";

import { validateEnvVars } from "@yin/common";

const schema = z.object({});

const env = validateEnvVars(schema);

export { env };

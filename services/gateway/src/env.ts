import { z } from "zod";

import { validateEnvVars } from "@yin/common";

const env = validateEnvVars(z.object({}));

export { env };

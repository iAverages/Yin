import { validateEnvVars } from "@yin/common";
import { z } from "zod";

const env = validateEnvVars(z.object({}));

export { env };

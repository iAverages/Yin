import { envWithPrefix, validateEnvVars } from "@yin/common";
import { z } from "zod";

const schema = envWithPrefix("YIN_WORKER", {
    GRPC_PORT: z.string(),
});

const env = validateEnvVars(schema);

export { env };

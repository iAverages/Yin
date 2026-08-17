import { createEnv } from "@t3-oss/env-core";
import { z } from "zod";

export const env = createEnv({
    clientPrefix: "VITE_",
    client: {
        VITE_API_URL: z.url().default("http://api.yin.localhost"),
        VITE_AUTH_URL: z.url().default("http://auth.yin.localhost"),
    },
    runtimeEnv: import.meta.env,
    emptyStringAsUndefined: true,
});

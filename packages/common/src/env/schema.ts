import { z } from "zod";

export const globalSchema = z.object({
    YIN_DISCORD_TOKEN: z.string(),
    NODE_ENV: z.enum(["development", "test", "production"]).default("development"),
    YIN_DEBUG: z.string().optional(),
    K3S_POD_NAME: z.string().default("local"),

    // Server specific but are shared
    YIN_WORKER_GRPC_PORT: z.string(),
    YIN_DATABASE_GRPC_PORT: z.string(),
});

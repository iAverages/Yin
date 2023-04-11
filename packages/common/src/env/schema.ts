import { z } from "zod";

export const globalSchema = z.object({
    YIN_DISCORD_TOKEN: z.string(),
    NODE_ENV: z.enum(["development", "test", "production"]).default("development"),
    YIN_DEBUG: z.string().optional(),

    // Server specific but are shared
    YIN_WORKER_GRPC_PORT: z.string(),
});

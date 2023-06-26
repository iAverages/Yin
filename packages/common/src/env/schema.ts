import { z } from "zod";

const devDefault = (def: string) => (process.env.NODE_ENV === "production" ? z.string() : z.string().default(def));

export const globalSchema = z.object({
    NODE_ENV: z.enum(["development", "test", "production"]).default("development"),
    K3S_POD_NAME: z.string().default("local"),
    SENTRY_DSN: z.string().optional(),

    YIN_DISCORD_TOKEN: z.string(),
    YIN_DEBUG: z.string().optional(),
    YIN_PROMETHEUS_PORT: z.string().default("8080"),

    // Server specific but are shared
    YIN_WORKER_GRPC_HOST: devDefault("localhost:50051"),
    YIN_DATABASE_GRPC_HOST: devDefault("localhost:50052"),
});

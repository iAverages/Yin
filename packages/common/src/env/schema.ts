import { z } from "zod";

const devDefault = (def: string) => (process.env.NODE_ENV === "production" ? z.string() : z.string().default(def));

export const globalSchema = z.object({
    YIN_DISCORD_TOKEN: z.string(),
    NODE_ENV: z.enum(["development", "test", "production"]).default("development"),
    YIN_DEBUG: z.string().optional(),
    K3S_POD_NAME: z.string().default("local"),

    // Server specific but are shared
    YIN_WORKER_GRPC_HOST: devDefault("localhost:50051"),
    YIN_DATABASE_GRPC_HOST: devDefault("localhost:50052"),
});

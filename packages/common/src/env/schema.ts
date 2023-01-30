import { z } from "zod";

export const globalSchema = z.object({
    DISCORD_TOKEN: z.string(),
    NODE_ENV: z.enum(["development", "test", "production"]).default("development"),
});

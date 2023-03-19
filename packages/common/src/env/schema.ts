import { z } from "zod";

export const envWithPrefix = (prefix: string, schema: Record<string, Zod.ZodTypeAny>) => {
    const obj: Record<string, Zod.ZodTypeAny> = {};
    for (const [key, value] of Object.entries(schema)) {
        obj[`${prefix}_${key}`] = value;
    }

    return z.object(obj);
};

export const globalSchema = z.object({
    YIN_DISCORD_TOKEN: z.string(),
    NODE_ENV: z.enum(["development", "test", "production"]).default("development"),
});

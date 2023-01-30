import { globalSchema } from "./schema";
import type { z, ZodFormattedError } from "zod";

export const formatErrors = (errors: ZodFormattedError<Map<string, string>, string>) =>
    Object.entries(errors)
        .map(([name, value]) => {
            return value && "_errors" in value ? `${name}: ${value._errors.join(", ")}\n` : false;
        })
        .filter(Boolean);

export const validateEnvVars = (passedSchema: z.AnyZodObject) => {
    const schema = passedSchema.merge(globalSchema);
    const _serverEnv = schema.safeParse(process.env);

    if (!_serverEnv.success) {
        console.error("❌ Invalid environment variables:\n", ...formatErrors(_serverEnv.error.format()));
        throw new Error("Invalid environment variables");
    }
};

import { schema } from "./schema";
import type { ZodFormattedError } from "zod";

const _serverEnv = schema.safeParse(process.env);

export const formatErrors = (errors: ZodFormattedError<Map<string, string>, string>) =>
    Object.entries(errors)
        .map(([name, value]) => {
            return value && "_errors" in value ? `${name}: ${value._errors.join(", ")}\n` : false;
        })
        .filter(Boolean);

if (!_serverEnv.success) {
    console.error("❌ Invalid environment variables:\n", ...formatErrors(_serverEnv.error.format()));
    throw new Error("Invalid environment variables");
}

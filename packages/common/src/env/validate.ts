import { z, type ZodFormattedError } from "zod";

import { globalSchema } from "./schema";

export const formatErrors = (errors: ZodFormattedError<Map<string, string>, string>) =>
    Object.entries(errors)
        .map(([name, value]) => {
            return value && "_errors" in value ? `${name}: ${value._errors.join(", ")}\n` : false;
        })
        .filter(Boolean);

type DefaultEnv = z.infer<typeof globalSchema>;
// This is here so common package can still use global env vars
// it wont be able to access the env vars from other packages which is fine
let env = process.env as DefaultEnv;

export const validateEnvVars = <T extends z.AnyZodObject>(passedSchema: T) => {
    const schema = globalSchema.merge(passedSchema);
    const _serverEnv = schema.safeParse(process.env);

    if (!_serverEnv.success) {
        console.error("❌ Invalid environment variables:\n", ...formatErrors(_serverEnv.error.format()));
        throw new Error("Invalid environment variables");
    }

    env = _serverEnv.data as unknown as DefaultEnv;
    return _serverEnv.data;
};

export { env };

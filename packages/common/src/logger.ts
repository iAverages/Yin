import createLogger, { type LoggerOptions } from "pino";

import { env } from "./env";

const prodLoggerOptions: LoggerOptions = {
    level: env.YIN_DEBUG ? "debug" : "info",
};

const devLoggerOptions: LoggerOptions = {
    transport: {
        target: "pino-pretty",
    },
    level: "debug",
};

export const logger = createLogger(env.NODE_ENV === "production" ? prodLoggerOptions : devLoggerOptions);

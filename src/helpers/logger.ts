import * as fs from "fs";
import * as colors from "colors";
import axios from "axios";

const streamMap = new Map();

enum LogLevel {
    ERROR,
    INFO,
    SUCCESS,
    WARN,
    DEBUG,
}

const getDate = () => {
    const doubleDigit = (num: number) => (num < 10 ? `0${num}` : num);
    const today = new Date();
    const dd = today.getDate();
    const mm = today.getMonth() + 1;
    const yyyy = today.getFullYear();
    return `${doubleDigit(dd)}-${doubleDigit(mm)}-${yyyy}`;
};

const getLogLevelInfo = (level: LogLevel) => {
    const genObj = (prefix: string, color: Function, logPath?: string[]) => {
        return { prefix, color, logPath };
    };

    switch (level) {
        case LogLevel.ERROR:
            return genObj("[ERROR]", colors.red, [`./logs/`, `error-${getDate()}.log`]);
        case LogLevel.DEBUG:
            return genObj("[UWU]", colors.magenta);
        case LogLevel.INFO:
            return genObj("[INFO]", colors.cyan, [`./logs/`, `log-${getDate()}.log`]);
        case LogLevel.SUCCESS:
            return genObj("[SUCCESS]", colors.green, [`./logs/`, `log-${getDate()}.log`]);
        case LogLevel.WARN:
            return genObj("[WARN]", colors.yellow, [`./logs/`, `error-${getDate()}.log`]);
    }
};

/**
 * Prefix message with timestamp
 * @param {String} message
 */
const logger = async (level: LogLevel, message: string) => {
    const date = new Date().toISOString().replace(/T/, " ").replace(/\..+/, "");
    const logLevelInfo = getLogLevelInfo(level);
    const logMessage = `[${date}] ${logLevelInfo.prefix} ${message}`.replace("[0m", "");
    const logMessageColor = logMessage;
    // const logMessageColor = `[${date}] ${logLevelInfo.color(logLevelInfo.prefix)} ${message}`;
    if (level === LogLevel.ERROR || level === LogLevel.WARN) {
        console.error(logMessageColor);
    } else {
        console.log(logMessageColor);
    }
    if (logLevelInfo.logPath != null && streamMap.has(logLevelInfo.logPath)) {
        const fsStream = streamMap.get(logLevelInfo.logPath);
        fsStream.write(logMessage + "\n");
    } else if (logLevelInfo.logPath) {
        try {
            await fs.promises.access(logLevelInfo.logPath[0]);
        } catch (error) {
            fs.mkdirSync(logLevelInfo.logPath[0], { recursive: true });
        }
        const fsStream = fs.createWriteStream(logLevelInfo.logPath.join(""), { flags: "a" });
        streamMap.set(logLevelInfo.logPath, fsStream);
        fsStream.write(logMessage + "\n");
    }
};

/**
 * Log a debug message. Prefixed with a magenta [UWU]
 * @param {String} message
 */
export const uwu = (message: string) => {
    logger(LogLevel.DEBUG, message);
};

/**
 * Log a info message. Prefixed with a cyan [INFO]
 * @param {String} message
 */
export const info = (message: string) => {
    logger(LogLevel.INFO, message);
};

/**
 * Log a success message. Prefixed with a green [SUCCESS]
 * @param {String} message
 */
export const success = (message: string) => {
    logger(LogLevel.SUCCESS, message);
};

/**
 * Log a warn message. Prefixed with a yellow [WARN]
 * @param {String} message
 */
export const warn = (message: string) => {
    logger(LogLevel.WARN, message);
};

/**
 * Log an error message. Prefixed with [ERROR], whole message is in red.
 * @param {String} message
 */
export const error = (message: string, ping = false) => {
    logger(LogLevel.ERROR, message);
    if (ping && process.env.DISCORD_ERROR_WEBHOOK) {
        try {
            axios.post(process.env.DISCORD_ERROR_WEBHOOK, {
                content: `${
                    process.env.DISCORD_PING_ID && `<@${process.env.DISCORD_PING_ID}>`
                } ${message}`,
            });
        } catch (err) {
            logger(LogLevel.ERROR, `Error while sending message to webhook: ${err.message}`);
        }
    }
};

export default {
    info,
    error,
    success,
    warn,
    uwu,
    debug: uwu,
};

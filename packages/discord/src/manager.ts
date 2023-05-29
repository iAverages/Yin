import axios, { AxiosError, type Method } from "axios";
import z from "zod";

import { __env, consts, logger } from "@yin/common";

import { type RequestResponses, type UrlParts } from "./requestResponses";
import { type Routes } from "./routes";

// Can't reference self
type _JsonValue = string | number | boolean | null;
export type JsonValue = _JsonValue | Record<string, _JsonValue>;

type Props<T> = {
    url: T;
    urlParts: UrlParts<T>;
    method: Method;
    body?: Record<string, any>;
    queryParams?: Record<string, JsonValue>;
    schema: z.AnyZodObject | z.ZodArray<z.AnyZodObject>;
};

export const makeUrl = <T extends Routes>(url: string, parts: UrlParts<T>) => {
    let formattedUrl = url;
    for (const [key, value] of Object.entries(parts ?? {})) {
        formattedUrl = formattedUrl.replace(`{${key}}`, `${value}`);
    }
    return formattedUrl;
};

export const responseTypes = {
    SUCCESS: 0,
    DISCORD_ERROR: 1,
    VALIDATION_ERROR: 2,
    UNKNOWN_ERROR: 3,
} as const;

type ResponseTypes = typeof responseTypes;

type SuccessfulResponse<T> = {
    success: true;
    type: ResponseTypes["SUCCESS"];
    data: RequestResponses<T>;
};

type DiscordErrorResponse = {
    success: false;
    type: ResponseTypes["DISCORD_ERROR"];
    message: string;
    code: number;
    errors: Record<string, unknown>;
};

type UnknownErrorResponse = {
    success: false;
    type: ResponseTypes["UNKNOWN_ERROR"];
    message: string;
    name: string;
};

type ValidationErrorResponse = {
    success: false;
    type: ResponseTypes["VALIDATION_ERROR"];
    message: string;
    issues: z.ZodIssue[];
};

type RequestResponse<T> = DiscordErrorResponse | ValidationErrorResponse | SuccessfulResponse<T> | UnknownErrorResponse;

export const req = async <T extends Routes>(input: Props<T>): Promise<RequestResponse<T>> => {
    const { url, method, body, queryParams, schema, urlParts } = input;
    try {
        const { data } = await axios({
            url: consts.discord.api + makeUrl(url, urlParts),
            method,
            data: body,
            params: queryParams,
            headers: {
                Authorization: `Bot ${__env.YIN_DISCORD_TOKEN}`,
                "User-Agent": consts.rest.userAgent,
            },
        });
        logger.debug(`[REST][DISCORD] Response:`, data);
        const validated = schema.parse(data);
        return { success: true, type: responseTypes.SUCCESS, data: validated as RequestResponses<T> };
    } catch (err) {
        if (err instanceof AxiosError) {
            console.log(err);
            return {
                success: false,
                type: responseTypes.DISCORD_ERROR,
                message: err.response?.data.message ?? "No messaged provided from Discord",
                code: err.response?.data.code,
                errors: err.response?.data.errors,
            };
        } else if (err instanceof z.ZodError) {
            return {
                success: false,
                type: responseTypes.VALIDATION_ERROR,
                message: "Zod Validation Error",
                issues: err.issues,
            };
        } else {
            const _err = err as Error;
            console.log(err);
            return { success: false, type: responseTypes.UNKNOWN_ERROR, message: _err.message, name: _err.name };
        }
    }
};

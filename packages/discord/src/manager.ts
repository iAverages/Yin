import { RequestResponses, UrlParts } from "./requestResponses";
import { Routes } from "./routes";
import axios, { Method, AxiosError } from "axios";
import z from "zod";
import { __env, consts, CONSTS } from "@yin/common";

const API_URL = "https://discord.com/api/v10";

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

type SuccessfulResponse<T> = {
    success: true;
    data: RequestResponses<T>;
};

type DiscordErrorResponse = {
    success: false;
    _discordError: true;
    _zodError: false;
    message: string;
    code: number;
    errors: Record<string, unknown>;
};

type NonDiscordErrorResponse = {
    success: false;
    _discordError: false;
    _zodError: false;
    message: string;
    name: string;
};

type ValidationErrorResponse = {
    success: false;
    _discordError: false;
    _zodError: true;
    message: string;
    _zod: z.ZodIssue[];
};
type RequestResponse<T> =
    | DiscordErrorResponse
    | ValidationErrorResponse
    | SuccessfulResponse<T>
    | NonDiscordErrorResponse;

export const req = async <T extends Routes>(input: Props<T>): Promise<RequestResponse<T>> => {
    const { url, method, body, queryParams, schema, urlParts } = input;
    try {
        const { data } = await axios({
            url: API_URL + makeUrl(url, urlParts),
            method,
            data: body,
            params: queryParams,
            headers: {
                Authorization: `Bot ${__env.YIN_DISCORD_TOKEN}`,
                "User-Agent": consts.rest.userAgent,
            },
        });
        if (__env.YIN_DEBUG) {
            console.log("[REST][DISCORD] Response: ", data);
        }
        const validated = schema.parse(data);
        return { success: true, data: validated as RequestResponses<T> };
    } catch (err) {
        if (err instanceof AxiosError) {
            console.log(err);
            return {
                success: false,
                _discordError: true,
                _zodError: false,
                message: err.response?.data.message ?? "No messaged provided from Discord",
                code: err.response?.data.code,
                errors: err.response?.data.errors,
            };
        } else if (err instanceof z.ZodError) {
            return {
                success: false,
                _discordError: false,
                _zodError: true,
                message: "Zod Validation Error",
                _zod: err.issues,
            };
        } else {
            const _err = err as Error;
            console.log(err);
            return { success: false, _discordError: false, _zodError: false, message: _err.message, name: _err.name };
        }
    }
};

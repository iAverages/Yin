import { RequestResponses, UrlParts } from "./requestResponses";
import { Routes } from "./routes";
import axios, { Method } from "axios";
import z from "zod";

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
    schema: z.AnyZodObject;
};

export const makeUrl = <T extends Routes>(url: string, parts: UrlParts<T>) => {
    let formattedUrl = url;
    for (const [key, value] of Object.entries(parts ?? {})) {
        formattedUrl = formattedUrl.replace(`{${key}}`, `${value}`);
    }
    return formattedUrl;
};

export const req = async <T extends Routes>(input: Props<T>) => {
    const { url, method, body, queryParams, schema, urlParts } = input;
    const { data } = await axios({
        url: API_URL + makeUrl(url, urlParts),
        method,
        data: body,
        params: queryParams,
        headers: {
            Authorization: `Bot `,
            "User-Agent": "Yin Rest API Wrapper (https://github.com/iAverages/Yin, 0.0.1)",
        },
    });
    console.log("[DISCORD] Response: ", data);
    const validated = schema.parse(data);
    return validated as RequestResponses<T>;
};

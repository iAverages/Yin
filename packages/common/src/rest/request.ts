import { RequestResponses, UrlParts } from "./requestResponses";
import { Http, Routes } from "./routes";
import axios from "axios";
import z from "zod";
import { env } from "env";

const API_URL = "https://discord.com/api/v10";

// Can't reference self
type _JsonValue = string | number | boolean | null;
export type JsonValue = _JsonValue | Record<string, _JsonValue>;

type Props<T> = {
    url: T;
    urlParts: UrlParts<T>;
    method: Http;
    body?: Record<string, any>;
    queryParams?: Record<string, JsonValue>;
    schema: z.AnyZodObject;
};

export const makeUrl = <T extends Routes>(url: string, parts: UrlParts<T>) => {
    let formattedUrl = url;
    for (const [key, value] of Object.entries(parts)) {
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
            Authorization: `Bot ${env.DISCORD_TOKEN}`,
            "User-Agent": "Yin Rest API Wrapper (https://github.com/iAverages/Yin, 0.0.1)",
        },
    });
    const validated = schema.parse(data);
    return validated as RequestResponses<T>;
};

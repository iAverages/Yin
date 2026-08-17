import { createIsomorphicFn } from "@tanstack/solid-start";
import { getRequestHeader, setResponseHeader } from "@tanstack/solid-start/server";

import { authClient } from "./auth-client";

export const getSession = createIsomorphicFn()
    .server(async () => {
        setResponseHeader("cache-control", "private, no-store");

        const cookie = getRequestHeader("cookie");
        if (!cookie) {
            return null;
        }

        let setCookies: string[] = [];
        const { data, error } = await authClient.getSession({
            fetchOptions: {
                headers: { cookie },
                onResponse: ({ response }) => {
                    setCookies = response.headers.getSetCookie();
                },
            },
        });

        if (setCookies.length > 0) {
            setResponseHeader("set-cookie", setCookies);
        }

        if (error) {
            throw new Error(`Session validation failed: ${error.message}`);
        }

        return data;
    })
    .client(async () => {
        const { data, error } = await authClient.getSession();
        if (error) {
            throw new Error(`Session validation failed: ${error.message}`);
        }

        return data;
    });

// Read-only SSR fixtures. Browser mutations remain intercepted by each test.
import http from "node:http";
import { adminGuilds, adminRuntimes } from "./admin-fixture.ts";

const guilds = [
    {
        id: "103",
        name: "Night Shift",
        icon: null,
        botInstalled: true,
    },
    {
        id: "102",
        name: "Yin Community",
        icon: null,
        botInstalled: true,
    },
    {
        id: "101",
        name: "Art Collective",
        icon: null,
        botInstalled: false,
    },
];
// Per-test observations let browser tests detect duplicate/cancelled SSR fetches.
const observations = new Map();
http.createServer(async (request, response) => {
    response.setHeader("content-type", "application/json");
    if (request.url === "/health") return response.end("{}");
    if (request.url?.startsWith("/__requests/")) {
        return response.end(
            JSON.stringify(
                observations.get(request.url.slice("/__requests/".length)) ?? {
                    paths: [],
                    aborted: 0,
                },
            ),
        );
    }
    const cookies = request.headers.cookie ?? "";
    if (!cookies.includes("fixture-session=1")) {
        response.statusCode = 401;
        return response.end("Not signed in.");
    }
    if (request.method !== "GET") {
        response.statusCode = 405;
        return response.end("SSR fixtures are read-only.");
    }
    const observationId = cookies.match(/fixture-request-id=([a-z0-9-]+)/)?.[1];
    if (observationId) {
        const observation = observations.get(observationId) ?? { paths: [], aborted: 0 };
        observations.set(observationId, observation);
        observation.paths.push(request.url);
        response.on("close", () => {
            if (!response.writableEnded) observation.aborted++;
        });
    }
    if (request.url?.startsWith("/admin/")) {
        response.setHeader("cache-control", "private, no-store");
        if (!cookies.includes("fixture-admin=owner")) {
            response.statusCode = 404;
            return response.end("not found");
        }
        if (request.url === "/admin/access") return response.end("true");
        if (cookies.includes("fixture-admin-data=error")) {
            response.statusCode = 503;
            return response.end("Diagnostics unavailable. Try again.");
        }
        if (cookies.includes("fixture-admin-data=empty")) return response.end("[]");
        if (request.url === "/admin/guilds") return response.end(JSON.stringify(adminGuilds));
        if (request.url === "/admin/state") return response.end(JSON.stringify(adminRuntimes));
        response.statusCode = 404;
        return response.end("not found");
    }
    const specificDelay =
        request.url === "/guilds" ? /fixture-guilds-delay=(\d+)/ : /fixture-settings-delay=(\d+)/;
    const delay =
        cookies.match(specificDelay)?.[1] ?? cookies.match(/fixture-api-delay=(\d+)/)?.[1];
    if (delay) await new Promise((resolve) => setTimeout(resolve, Math.min(Number(delay), 5000)));
    if (request.url === "/guilds") {
        if (cookies.includes("fixture-guilds=error")) {
            response.statusCode = 403;
            return response.end("Sign in again.");
        }
        return response.end(
            JSON.stringify(cookies.includes("fixture-guilds=empty") ? [guilds[2]] : guilds),
        );
    }
    const match = request.url?.match(/^\/guilds\/(\d+)\/settings$/);
    const guild = guilds.find((guild) => guild.id === match?.[1]);
    if (!guild) {
        response.statusCode = 404;
        return response.end("Server not found.");
    }
    if (cookies.includes("fixture-settings=error")) {
        response.statusCode = 403;
        return response.end("You need Manage Server or Administrator permission.");
    }
    response.end(
        JSON.stringify({
            guild,
            commandPrefix: cookies.includes("fixture-values=custom") ? "?" : null,
            activePrefix: cookies.includes("fixture-values=custom") ? "?" : "!",
            translationLanguage: cookies.includes("fixture-values=custom") ? "ja" : null,
            activeTranslationLanguage: cookies.includes("fixture-values=custom") ? "ja" : "en",
            socialEmbeds: [{ platform: "twitter", enabled: true }],
            customCommands: [{ name: "rules", response: "Read #rules." }],
            ladderRules: [
                {
                    id: 1,
                    warningThreshold: 3,
                    windowSeconds: 2592000,
                    action: "timeout",
                    durationSeconds: 86400,
                },
            ],
        }),
    );
}).listen(33011, "127.0.0.1");

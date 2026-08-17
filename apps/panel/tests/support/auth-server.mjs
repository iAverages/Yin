// Isolated test service, never loaded by the application or production auth.
import http from "node:http";

const timestamp = "2026-01-01T00:00:00.000Z";
const session = {
    user: {
        id: "fixture",
        name: "Dan",
        email: "dan@example.com",
        image: null,
        emailVerified: true,
        createdAt: timestamp,
        updatedAt: timestamp,
    },
    session: {
        id: "fixture",
        userId: "fixture",
        token: "fixture",
        expiresAt: "2099-01-01T00:00:00.000Z",
        createdAt: timestamp,
        updatedAt: timestamp,
    },
};

http.createServer((request, response) => {
    response.setHeader("content-type", "application/json");
    response.setHeader("access-control-allow-origin", "http://127.0.0.1:3010");
    response.setHeader("access-control-allow-credentials", "true");
    response.setHeader("access-control-allow-methods", "GET, POST, OPTIONS");
    response.setHeader("access-control-allow-headers", "content-type");
    if (request.method === "OPTIONS") return response.end();
    if (request.url === "/health") return response.end("{}");
    if (request.url?.startsWith("/api/auth/get-session")) {
        return response.end(
            JSON.stringify(request.headers.cookie?.includes("fixture-session=1") ? session : null),
        );
    }
    if (request.url === "/api/auth/sign-out") {
        response.setHeader(
            "set-cookie",
            "fixture-session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax",
        );
        return response.end(JSON.stringify({ success: true }));
    }
    response.statusCode = 404;
    response.end("{}");
}).listen(33010, "127.0.0.1");

import { defineConfig } from "@playwright/test";

export default defineConfig({
    testDir: "./tests",
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: 2,
    reporter: "list",
    use: {
        headless: true,
        userAgent:
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/152.0.0.0 Safari/537.36",
        baseURL: "http://127.0.0.1:3010",
        viewport: { width: 1440, height: 1000 },
        trace: "retain-on-failure",
        screenshot: "only-on-failure",
        launchOptions: { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE },
    },
    webServer: [
        {
            command: "node tests/support/api-server.mjs",
            url: "http://127.0.0.1:33011/health",
            reuseExistingServer: false,
        },
        {
            command: "node tests/support/auth-server.mjs",
            url: "http://127.0.0.1:33010/health",
            reuseExistingServer: false,
        },
        {
            command: "pnpm exec vite dev --host 127.0.0.1 --port 3010 --strictPort",
            url: "http://127.0.0.1:3010/login",
            reuseExistingServer: false,
            env: {
                VITE_AUTH_URL: "http://127.0.0.1:33010",
                VITE_API_URL: "http://127.0.0.1:33011",
            },
        },
    ],
});

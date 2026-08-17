import { expect, test, type BrowserContext, type Page } from "@playwright/test";

import { adminGuilds, adminRuntimes } from "./support/admin-fixture.ts";

const signIn = async (context: BrowserContext, owner = true) => {
    await context.addCookies([
        { name: "fixture-session", value: "1", url: "http://127.0.0.1:3010" },
        {
            name: "fixture-admin",
            value: owner ? "owner" : "ordinary",
            url: "http://127.0.0.1:3010",
        },
    ]);
};

const mockApi = async (page: Page) => {
    await page.route("http://127.0.0.1:33011/**", (route) => {
        const path = new URL(route.request().url()).pathname;
        if (path === "/admin/access") return route.fulfill({ json: true });
        if (path === "/admin/state") return route.fulfill({ json: adminRuntimes });
        if (path === "/admin/guilds") return route.fulfill({ json: adminGuilds });
        if (path === "/guilds") return route.fulfill({ json: [] });
        return route.fulfill({ status: 404, body: "not found" });
    });
};

test("signed-out and non-owner requests cannot render diagnostics", async ({
    page,
    context,
    request,
}) => {
    await page.goto("/admin");
    await expect(page).toHaveURL(/\/login$/);
    await signIn(context, false);
    const response = await page.goto("/admin");
    expect(response?.status()).toBe(404);
    await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Bot overview" })).toHaveCount(0);
    expect(await response!.text()).not.toContain("987654321012345678");
    for (const path of ["/admin/access", "/admin/state", "/admin/guilds"]) {
        const api = await request.get(`http://127.0.0.1:33011${path}`, {
            headers: { cookie: "fixture-session=1" },
        });
        expect(api.status()).toBe(404);
    }
});

test("owner sees SSR diagnostics without JavaScript and no navigation entry", async ({
    browser,
}) => {
    const context = await browser.newContext({ javaScriptEnabled: false });
    try {
        await signIn(context);
        const page = await context.newPage();
        const response = await page.goto("/admin", { waitUntil: "domcontentloaded" });
        expect(response?.headers()["cache-control"]).toContain("no-store");
        await expect(page.getByRole("heading", { name: "Bot overview" })).toBeVisible();
        await expect(page.getByRole("table", { name: /Shard statuses/ })).toContainText("42 ms");
        await expect(page.getByRole("table", { name: /Shard statuses/ })).toContainText("resuming");
        await expect(page.getByRole("table", { name: "Bot guild directory" })).toContainText(
            "9007199254740993",
        );
        await expect(page.getByRole("table", { name: "Bot guild directory" })).toContainText(
            "Elsewhere",
        );
        await expect(page.getByRole("button", { name: "Refresh state" })).toBeDisabled();
        await expect(page.locator('a[href^="/admin"]')).toHaveCount(0);
        await page.goto("/");
        await expect(page.locator('a[href^="/admin"]')).toHaveCount(0);
    } finally {
        await context.close();
    }
});

test("guild search preserves snowflakes and supports empty matches", async ({ page, context }) => {
    await signIn(context);
    await mockApi(page);
    await page.goto("/admin");
    const search = page.getByRole("searchbox", { name: "Search guilds" });
    const directory = page.getByRole("table", { name: "Bot guild directory" });
    await expect(search).toBeEnabled();
    await search.fill("ELSEWHERE");
    await expect(directory.getByRole("row")).toHaveCount(2);
    await expect(directory).toContainText("9007199254740993");
    await search.fill("9007199254740993");
    await expect(directory).toContainText("Elsewhere");
    await search.fill("not a real guild");
    await expect(directory).toContainText("No guilds match your search.");
    await search.clear();
    await expect(directory.getByRole("row")).toHaveCount(4);
    await expect(directory).toContainText("Metadata pending");
    await page.getByRole("button", { name: "Account menu" }).click();
    await expect(page.getByRole("menuitem", { name: /admin/i })).toHaveCount(0);
});

test("refresh handles stale, stopped and failed diagnostics without implying live gateways", async ({
    page,
    context,
}) => {
    await signIn(context);
    await mockApi(page);
    await page.goto("/admin");
    await expect(page.getByRole("button", { name: "Refresh state" })).toBeEnabled();
    for (const status of ["stale", "stopped"] as const) {
        await page.route("http://127.0.0.1:33011/admin/state", (route) =>
            route.fulfill({
                json: [{ ...adminRuntimes[0], status, ageSeconds: 90 }],
            }),
        );
        await page.getByRole("button", { name: "Refresh state" }).click();
        await expect(
            page.getByText(status === "stale" ? "Stale" : "Stopped", { exact: true }),
        ).toBeVisible();
        await expect(page.getByText(/Historical snapshot/)).toBeVisible();
        await expect(page.getByRole("columnheader", { name: "Last gateway status" })).toBeVisible();
    }
    await page.route("http://127.0.0.1:33011/admin/state", (route) =>
        route.fulfill({ status: 503, body: "Diagnostics unavailable." }),
    );
    await page.getByRole("button", { name: "Refresh state" }).click();
    // Server errors are retried with backoff before the failure is shown.
    await expect(page.getByRole("alert")).toContainText("Diagnostics unavailable.", {
        timeout: 15_000,
    });
    await expect(page.getByText(/Displayed diagnostics may be out of date/)).toBeVisible();
    await page.route("http://127.0.0.1:33011/admin/state", (route) =>
        route.fulfill({ json: adminRuntimes }),
    );
    await page.getByRole("button", { name: "Try again" }).click();
    await expect(page.getByText("Reporting", { exact: true })).toBeVisible();
    await expect(page.getByRole("alert")).toHaveCount(0);
});

test("snapshots age into stale even when no new response arrives", async ({ page, context }) => {
    await signIn(context);
    await mockApi(page);
    await page.goto("/admin");
    await expect(page.getByRole("button", { name: "Refresh state" })).toBeEnabled();
    await page.clock.install();
    await page.route("http://127.0.0.1:33011/admin/state", (route) => route.abort());
    await page.clock.fastForward(46_000);
    await expect(page.getByText("Stale", { exact: true })).toBeVisible();
    await expect(page.getByText(/Historical snapshot/)).toBeVisible();
});

test("empty diagnostics explain what to check and fit a mobile viewport", async ({
    page,
    context,
}) => {
    await signIn(context);
    await context.addCookies([
        { name: "fixture-admin-data", value: "empty", url: "http://127.0.0.1:3010" },
    ]);
    await mockApi(page);
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/admin");
    await expect(page.getByText(/No bot snapshots in the last 24 hours/)).toBeVisible();
    await expect(page.getByText(/No guild memberships recorded/)).toBeVisible();
    await expect(page.getByRole("button", { name: "Refresh state" })).toBeEnabled();
    await page.getByRole("button", { name: "Refresh state" }).click();
    await expect(page.getByText("Reporting", { exact: true })).toBeVisible();
    expect(
        await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
    ).toBe(true);
});

test("initial data failures stay local and can be retried", async ({ page, context }) => {
    await signIn(context);
    await context.addCookies([
        { name: "fixture-admin-data", value: "error", url: "http://127.0.0.1:3010" },
    ]);
    await mockApi(page);
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.route("http://127.0.0.1:33011/admin/state", (route) =>
        route.fulfill({ status: 503, body: "Diagnostics unavailable. Try again." }),
    );
    await page.goto("/admin");
    const runtime = page.getByRole("region", { name: "Gateway & runtime" });
    await expect(page.getByRole("heading", { name: "Bot overview" })).toBeVisible();
    // Server errors are retried with backoff before the failure is shown.
    await expect(runtime.getByRole("alert")).toContainText("Diagnostics unavailable.", {
        timeout: 15_000,
    });
    await expect(runtime.getByRole("button", { name: "Try again" })).toBeEnabled({
        timeout: 15_000,
    });
    await page.route("http://127.0.0.1:33011/admin/state", (route) =>
        route.fulfill({ json: adminRuntimes }),
    );
    await runtime.getByRole("button", { name: "Try again" }).click();
    await expect(runtime.getByText("Reporting", { exact: true })).toBeVisible();
    // Solid streams the expected SSR resource errors; hydration and other errors
    // must not be introduced by rendering those failures.
    expect(errors.filter((message) => message !== "Diagnostics unavailable. Try again.")).toEqual(
        [],
    );
});

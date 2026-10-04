import { expect, test, type Page } from "@playwright/test";

import type { GuildSettings, ManagedGuild } from "../src/lib/api-client";

const guilds: ManagedGuild[] = [
    {
        id: "103",
        name: "Night Shift",
        icon: null,
        permissions: "8",
        canManage: true,
        botInstalled: true,
    },
    {
        id: "102",
        name: "Yin Community",
        icon: null,
        permissions: "32",
        canManage: true,
        botInstalled: true,
    },
    {
        id: "101",
        name: "Art Collective",
        icon: null,
        permissions: "32",
        canManage: true,
        botInstalled: false,
    },
];

const settings = (guildId: string): GuildSettings => {
    return {
        guild: guilds.find((guild) => guild.id === guildId)!,
        commandPrefix: null,
        activePrefix: "!",
        translationLanguage: null,
        activeTranslationLanguage: "en",
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
    };
};

const mockGuilds = async (page: Page) => {
    await page.route("http://127.0.0.1:33011/**", (route) => {
        const path = new URL(route.request().url()).pathname;
        if (path === "/guilds") return route.fulfill({ json: guilds });
        const match = path.match(/^\/guilds\/(\d+)\/settings$/);
        if (match) return route.fulfill({ json: settings(match[1]) });
        return route.fulfill({ status: 404, body: "Unexpected test request" });
    });
};

test.beforeEach(async ({ page, context }) => {
    await context.addCookies([
        { name: "fixture-session", value: "1", url: "http://127.0.0.1:3010" },
    ]);
    await mockGuilds(page);
});

test("the server renders actual dashboard content with JavaScript disabled", async ({
    browser,
}) => {
    const context = await browser.newContext({
        javaScriptEnabled: false,
        viewport: { width: 1440, height: 1000 },
    });
    await context.addCookies([
        { name: "fixture-session", value: "1", url: "http://127.0.0.1:3010" },
    ]);
    const page = await context.newPage();
    try {
        await page.goto("/", { waitUntil: "domcontentloaded" });
        const sidebar = page.getByRole("complementary", { name: "Dashboard sidebar" });
        await expect(sidebar.getByRole("link", { name: "Servers", exact: true })).toBeVisible();
        await expect(sidebar.getByRole("button", { name: "Account menu" })).toContainText("Dan");
        await expect(sidebar.getByRole("link", { name: "Add a server" })).toBeVisible();
        await expect(page.getByRole("link", { name: "Manage Night Shift" })).toBeVisible();
        await expect(page.getByRole("status")).toHaveCount(0);
        await context.addCookies([
            { name: "fixture-values", value: "custom", url: "http://127.0.0.1:3010" },
        ]);
        await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
        await expect(page.getByRole("heading", { name: "Night Shift", exact: true })).toBeVisible();
        await expect(page.getByRole("textbox", { name: /^Command prefix/ })).toHaveValue("?");
        await expect(page.getByRole("textbox", { name: /^Translation language/ })).toHaveValue(
            "ja",
        );
        await expect(
            page.getByRole("region", { name: "Custom commands", exact: true }),
        ).toContainText("Read #rules.");
        await expect(page.getByRole("status")).toHaveCount(0);
        await page.goto("/guilds/103/moderation", { waitUntil: "domcontentloaded" });
        await expect(page.getByRole("region", { name: "Punishment ladder" })).toContainText(
            "3 warnings in 30 days",
        );
        await expect(page.getByRole("status")).toHaveCount(0);
    } finally {
        await context.close();
    }
});

test("server-rendered content does not need the application bundle", async ({ page }) => {
    const apiRequests: string[] = [];
    page.on("request", (request) => {
        if (request.url().includes(":33011/")) apiRequests.push(request.url());
    });
    await page.route("**/*", (route) =>
        route.request().resourceType() === "script" ? route.abort() : route.fallback(),
    );
    // No application code can fetch or render this content.
    for (const path of ["/", "/guilds/103", "/guilds/103/moderation"]) {
        await page.goto(path, { waitUntil: "domcontentloaded" });
        if (path === "/") {
            await expect(page.getByRole("link", { name: "Manage Night Shift" })).toBeVisible();
        } else {
            await expect(
                page.getByRole("heading", { name: "Night Shift", exact: true }),
            ).toBeVisible();
            await expect(
                page.getByRole("button", {
                    name: path.endsWith("moderation") ? "Add ladder rule" : "Save general",
                    exact: true,
                }),
            ).toBeDisabled();
        }
    }
    expect(apiRequests).toEqual([]);
});

test("SSR values are request-scoped and hydrate without fetching again", async ({ browser }) => {
    const contexts = await Promise.all([browser.newContext(), browser.newContext()]);
    try {
        await Promise.all(
            contexts.map(async (context, index) => {
                await context.addCookies([
                    { name: "fixture-session", value: "1", url: "http://127.0.0.1:3010" },
                    {
                        name: "fixture-values",
                        value: index === 0 ? "custom" : "default",
                        url: "http://127.0.0.1:3010",
                    },
                ]);
                const page = await context.newPage();
                const apiRequests: string[] = [];
                page.on("request", (request) => {
                    if (request.url().includes(":33011/")) apiRequests.push(request.url());
                });
                const response = await page.goto("http://127.0.0.1:3010/guilds/103", {
                    waitUntil: "domcontentloaded",
                });
                expect(response?.headers()["cache-control"]).toContain("no-store");
                const prefix = page.getByRole("textbox", { name: /^Command prefix/ });
                await expect(prefix).toBeEnabled();
                await expect(prefix).toHaveValue(index === 0 ? "?" : "");
                await expect(
                    page.getByRole("textbox", { name: /^Translation language/ }),
                ).toHaveValue(index === 0 ? "ja" : "");
                expect(apiRequests).toEqual([]);
            }),
        );
    } finally {
        await Promise.all(contexts.map((context) => context.close()));
    }
});

test("a slow initial API response still renders content without JavaScript", async ({
    browser,
}) => {
    const context = await browser.newContext({ javaScriptEnabled: false });
    await context.addCookies([
        { name: "fixture-session", value: "1", url: "http://127.0.0.1:3010" },
        { name: "fixture-api-delay", value: "1500", url: "http://127.0.0.1:3010" },
    ]);
    const page = await context.newPage();
    try {
        await page.goto("/guilds/103/moderation", { waitUntil: "domcontentloaded" });
        await expect(page.getByRole("region", { name: "Punishment ladder" })).toContainText(
            "3 warnings in 30 days",
        );
        await expect(page.getByRole("button", { name: "Select server" })).toContainText(
            "Night Shift",
        );
        await expect(page.getByRole("status")).toHaveCount(0);
    } finally {
        await context.close();
    }
});

for (const moderation of [false, true]) {
    test(`client navigation keeps the sidebar visible while loading ${moderation ? "moderation" : "settings"}`, async ({
        page,
    }) => {
        await page.goto(moderation ? "/guilds/103/moderation" : "/", {
            waitUntil: "domcontentloaded",
        });
        await expect(page.getByRole("button", { name: "Select server" })).toBeEnabled();
        let release!: () => void;
        const gate = new Promise<void>((resolve) => {
            release = resolve;
        });
        await page.route("**/guilds/102/settings", async (route) => {
            await gate;
            await route.fulfill({ json: settings("102") });
        });
        await page.getByRole("button", { name: "Select server" }).click();
        await page.getByRole("link", { name: "Yin Community", exact: true }).click();
        const loading = page.getByRole("status", {
            name: moderation ? "Loading moderation" : "Loading server settings",
            exact: true,
        });
        await expect(loading).toBeVisible();
        await expect(page.getByRole("button", { name: "Account menu" })).toBeVisible();
        await expect(
            page
                .getByRole("navigation", { name: "Dashboard navigation", exact: true })
                .getByRole("link", { name: "Servers", exact: true }),
        ).toBeVisible();
        await expect(
            page.getByRole("button", {
                name: moderation ? "Add ladder rule" : "Save general",
                exact: true,
            }),
        ).toHaveCount(0);
        release();
        await expect(
            page.getByRole("heading", { name: "Yin Community", exact: true }),
        ).toBeVisible();
        await expect(loading).toBeHidden();
        await expect(page).toHaveURL(moderation ? "/guilds/102/moderation" : "/guilds/102");
    });
}

test("moderation has its own route and shares the server settings cache", async ({ page }) => {
    let settingsRequests = 0;
    page.on("request", (request) => {
        if (request.url().endsWith(":33011/guilds/103/settings")) settingsRequests++;
    });
    await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("button", { name: "Save general", exact: true })).toBeEnabled();
    await expect(page.getByRole("heading", { name: "Punishment ladder" })).toHaveCount(0);
    const navigation = page.getByRole("navigation", { name: "Dashboard navigation", exact: true });
    await navigation.getByRole("link", { name: "Moderation", exact: true }).click();
    await expect(page).toHaveURL("/guilds/103/moderation");
    await expect(page.getByRole("region", { name: "Punishment ladder" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "General", exact: true })).toHaveCount(0);
    await expect(navigation.getByRole("link", { name: "Moderation", exact: true })).toHaveAttribute(
        "aria-current",
        "page",
    );
    await expect(
        navigation.getByRole("link", { name: "Server settings", exact: true }),
    ).not.toHaveAttribute("aria-current", "page");
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText("Moderation");
    await navigation.getByRole("link", { name: "Server settings", exact: true }).click();
    await expect(page.getByRole("region", { name: "General", exact: true })).toBeVisible();
    expect(settingsRequests).toBe(0);
});

test("mobile moderation navigation closes the sheet and keeps the table usable", async ({
    page,
}) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
    await page.getByRole("button", { name: "Open navigation" }).click();
    const drawer = page.getByRole("dialog", { name: "Dashboard navigation" });
    await drawer.getByRole("link", { name: "Moderation", exact: true }).click();
    await expect(page).toHaveURL("/guilds/103/moderation");
    await expect(drawer).toBeHidden();
    await page.route("**/guilds/103/ladder-rules/1", (route) =>
        route.fulfill({ json: { ...settings("103"), ladderRules: [] } }),
    );
    await page
        .getByRole("region", { name: "Punishment ladder" })
        .getByRole("button", { name: "Remove", exact: true })
        .click();
    await expect(
        page.getByRole("cell", { name: "No punishment ladder rules configured." }),
    ).toBeVisible();
    await expect(page.getByRole("region", { name: /^Notifications/ })).toContainText(
        "Punishment ladder rule removed.",
    );
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(
        true,
    );
});

test("SSR settings errors keep navigation available and can be retried", async ({
    page,
    context,
}) => {
    await context.addCookies([
        { name: "fixture-settings", value: "error", url: "http://127.0.0.1:3010" },
    ]);
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.route("**/guilds/103/settings", (route) =>
        route.fulfill({ status: 403, body: "You need Manage Server or Administrator permission." }),
    );
    await page.goto("/guilds/103/moderation", { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("button", { name: "Toggle sidebar" })).toBeEnabled();
    const alert = page.getByRole("main").getByRole("alert");
    await expect(alert).toContainText("You need Manage Server");
    await expect(page.getByRole("button", { name: "Account menu" })).toBeVisible();
    await page.unroute("**/guilds/103/settings");
    await alert.getByRole("button", { name: "Try again" }).click();
    await expect(page.getByRole("region", { name: "Punishment ladder" })).toBeVisible();
    await expect(alert).toBeHidden();
    // Solid Query also serializes its rejected SSR promise before hydration.
    // Only the deliberate API failure is allowed, never a hydration/runtime error.
    expect(
        errors.filter(
            (message) => message !== "You need Manage Server or Administrator permission.",
        ),
    ).toEqual([]);
});

test("shared sidebar, inset layout, guild switching and cached navigation", async ({ page }) => {
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    let requests = 0;
    page.on("request", (request) => {
        if (request.url() === "http://127.0.0.1:33011/guilds") requests++;
    });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    const servers = page.getByRole("region", { name: "Servers", exact: true });
    await expect(servers.getByRole("link")).toHaveText([
        /Night Shift/,
        /Yin Community/,
        /Art Collective.*Add Yin/,
    ]);
    await expect(servers).not.toContainText("Yin installed");
    await expect(servers.getByRole("link", { name: "Add Yin to Art Collective" })).toHaveAttribute(
        "href",
        /guild_id=101$/,
    );
    const installed = servers.getByRole("link", { name: "Manage Night Shift" });
    const absent = servers.getByRole("link", { name: "Add Yin to Art Collective" });
    await expect(installed).not.toContainText("→");
    await expect(installed.locator("[aria-hidden=true]").last()).toHaveCSS("mask-image", /url\(/);
    await expect(installed).toHaveCSS("background-color", "rgb(28, 31, 38)");
    await expect(absent).toHaveCSS("background-color", "rgb(12, 13, 16)");
    const sidebar = page.getByRole("complementary", { name: "Dashboard sidebar" });
    await expect(sidebar.getByRole("link", { name: "Add a server" })).toBeVisible();
    await expect(page.getByText("Session", { exact: true })).toHaveCount(0);

    // Collapsed state survives route navigation only if the layout stays mounted.
    await page.getByRole("button", { name: "Toggle sidebar" }).click();
    await installed.click();
    await expect(page.getByRole("heading", { name: "Night Shift", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Toggle sidebar" })).toHaveAttribute(
        "aria-expanded",
        "false",
    );
    await page.getByRole("button", { name: "Toggle sidebar" }).click();
    const selector = sidebar.getByRole("button", { name: "Select server" });
    await expect(selector).toContainText("Night Shift");
    await selector.click();
    await page.keyboard.press("Home");
    await page.keyboard.press("ArrowDown");
    await expect(page.getByRole("link", { name: "Night Shift", exact: true })).toBeFocused();
    await page.keyboard.press("ArrowDown");
    await expect(page.getByRole("link", { name: "Yin Community", exact: true })).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page).toHaveURL("/guilds/102");
    await expect(page.getByRole("heading", { name: "Yin Community", exact: true })).toBeVisible();
    await expect(selector).toContainText("Yin Community");
    await sidebar.getByRole("link", { name: "Servers", exact: true }).click();
    await expect(servers).toBeVisible();
    // SSR hydrated the shared query cache; navigation must not fetch it again.
    expect(requests).toBe(0);
    expect(errors).toEqual([]);
});

for (const mobile of [false, true]) {
    test(`guild search filters installed servers and supports keyboard selection (${mobile ? "mobile" : "desktop"})`, async ({
        page,
    }) => {
        if (mobile) await page.setViewportSize({ width: 390, height: 844 });
        await page.goto("/", { waitUntil: "domcontentloaded" });
        await expect(page.getByRole("link", { name: "Manage Night Shift" })).toBeVisible();
        if (mobile) await page.getByRole("button", { name: "Open navigation" }).click();
        await page.getByRole("button", { name: "Select server" }).click();
        const selector = page.getByRole("dialog", { name: "Select server", exact: true });
        const search = selector.getByRole("searchbox", { name: "Search servers" });
        const options = selector.getByRole("navigation", { name: "Servers with Yin" });
        await expect(search).toBeFocused();
        await expect(options.getByRole("link")).toHaveCount(2);
        await expect(selector).not.toContainText("Art Collective");
        await search.fill("  yIN  ");
        await expect(options.getByRole("link")).toHaveCount(1);
        await expect(
            options.getByRole("link", { name: "Yin Community", exact: true }),
        ).toBeVisible();
        await search.fill("Art");
        await expect(options.getByRole("status")).toHaveText("No matching servers.");
        await expect(options.getByRole("link")).toHaveCount(0);
        await search.fill("night");
        await search.press("ArrowDown");
        await expect(options.getByRole("link", { name: "Night Shift", exact: true })).toBeFocused();
        await page.keyboard.press("Enter");
        await expect(page.getByRole("heading", { name: "Night Shift", exact: true })).toBeVisible();
        await expect(selector).toBeHidden();
        if (mobile) {
            await expect(page.getByRole("dialog", { name: "Dashboard navigation" })).toBeHidden();
            await page.getByRole("button", { name: "Open navigation" }).click();
        }
        await page.getByRole("button", { name: "Select server" }).click();
        await expect(search).toHaveValue("");
        await expect(options.getByRole("link")).toHaveCount(2);
    });
}

test("selector handles no installed servers without hiding dashboard install links", async ({
    page,
    context,
}) => {
    await context.addCookies([
        { name: "fixture-guilds", value: "empty", url: "http://127.0.0.1:3010" },
    ]);
    await page.route("**/guilds", (route) => route.fulfill({ json: [guilds[0]] }));
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("link", { name: "Add Yin to Art Collective" })).toBeVisible();
    await page.getByRole("button", { name: "Select server" }).click();
    const selector = page.getByRole("dialog", { name: "Select server", exact: true });
    await expect(selector.getByRole("status")).toHaveText("No servers with Yin.");
    await expect(selector.getByRole("navigation").getByRole("link")).toHaveCount(0);
    await expect(selector.getByRole("link", { name: "All servers" })).toBeVisible();
});

test("popovers animate in and out and sidebar collapse closes them", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("link", { name: "Manage Night Shift" })).toBeVisible();
    const animations = await page.evaluateHandle(() => {
        const names: string[] = [];
        document.addEventListener("animationstart", (event) => names.push(event.animationName));
        return names;
    });
    for (const [index, name] of ["Select server", "Account menu"].entries()) {
        const trigger = page.getByRole("button", { name, exact: true });
        await trigger.click();
        const popup =
            name === "Select server"
                ? page.getByRole("dialog", { name, exact: true })
                : page.getByRole("menu", { name, exact: true });
        await expect(popup).toHaveCSS("animation-name", "popover-in");
        await expect
            .poll(
                async () =>
                    (await animations.jsonValue()).filter((name) => name === "popover-in").length,
            )
            .toBe(index + 1);
        await page.keyboard.press("Escape");
        await expect(popup).toBeHidden();
        await expect(trigger).toBeFocused();
    }
    await expect
        .poll(() => animations.jsonValue())
        .toEqual(["popover-in", "popover-out", "popover-in", "popover-out"]);
    await page.getByRole("button", { name: "Account menu" }).click();
    await page.getByRole("button", { name: "Toggle sidebar" }).click();
    await expect(page.getByRole("menu", { name: "Account menu", exact: true })).toBeHidden();
    // Structural ID is also the sidebar toggle's aria-controls target.
    const sidebar = page.locator("#dashboard-sidebar");
    await expect(sidebar).toHaveCSS("transition-property", "width");
    await expect(sidebar).toHaveCSS("width", "0px");
    await expect(sidebar).toHaveAttribute("inert", "");
    await page.getByRole("button", { name: "Toggle sidebar" }).click();
    await expect(sidebar).toHaveCSS("width", "256px");
    await expect(sidebar).not.toHaveAttribute("inert", "");
    await page.emulateMedia({ reducedMotion: "reduce" });
    await expect(sidebar).toHaveCSS("transition-duration", "1e-05s");
    await animations.dispose();
});

test("Ctrl+B toggles the fixed sidebar and removes the collapsed content frame", async ({
    page,
}) => {
    await page.setViewportSize({ width: 1440, height: 720 });
    await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("heading", { name: "Night Shift", exact: true })).toBeVisible();
    const sidebar = page.locator("#dashboard-sidebar");
    const view = page.locator("#dashboard-view");
    const toggle = page.getByRole("button", { name: "Toggle sidebar" });
    await expect(toggle).toBeEnabled();
    await expect(toggle).toHaveAttribute("aria-keyshortcuts", "Control+b");
    await expect(sidebar).toHaveCSS("position", "fixed");
    await expect(view).toHaveCSS("margin-left", "256px");
    await expect(view).toHaveCSS("border-radius", "12px");
    await page.mouse.wheel(0, 800);
    await expect.poll(() => page.evaluate(() => scrollY)).toBeGreaterThan(100);
    expect((await sidebar.boundingBox())!.y).toBe(0);
    expect((await sidebar.boundingBox())!.height).toBe(720);
    await expect(sidebar.getByRole("button", { name: "Account menu" })).toBeInViewport();
    await expect(sidebar.getByRole("button", { name: "Select server" })).toBeInViewport();

    await page.keyboard.press("Control+b");
    await expect(toggle).toHaveAttribute("aria-expanded", "false");
    await expect(sidebar).toHaveCSS("width", "0px");
    await expect(view).toHaveCSS("margin", "0px");
    await expect(view).toHaveCSS("border-radius", "0px");
    await expect(view).toHaveCSS("box-shadow", "none");
    expect((await view.boundingBox())!.x).toBe(0);
    expect((await view.boundingBox())!.width).toBe(1440);
    await page.keyboard.press("Control+b");
    await expect(toggle).toHaveAttribute("aria-expanded", "true");
    await expect(view).toHaveCSS("margin-left", "256px");
    await expect(sidebar).not.toHaveAttribute("inert", "");

    // Holding the shortcut must not oscillate the sidebar.
    await page.keyboard.down("Control");
    await page.keyboard.down("b");
    await expect(toggle).toHaveAttribute("aria-expanded", "false");
    await page.keyboard.down("b");
    await expect(toggle).toHaveAttribute("aria-expanded", "false");
    await page.keyboard.up("b");
    await page.keyboard.up("Control");
});

test("Ctrl+B toggles mobile navigation without toggling desktop state", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/", { waitUntil: "domcontentloaded" });
    const trigger = page.getByRole("button", { name: "Open navigation" });
    const drawer = page.getByRole("dialog", { name: "Dashboard navigation" });
    await expect(trigger).toBeEnabled();
    await page.keyboard.press("Control+b");
    await expect(drawer).toBeVisible();
    await page.keyboard.press("Control+b");
    await expect(drawer).toBeHidden();
    await expect(trigger).toBeFocused();
    await page.setViewportSize({ width: 1440, height: 1000 });
    await expect(page.getByRole("button", { name: "Toggle sidebar" })).toHaveAttribute(
        "aria-expanded",
        "true",
    );
});

test("refresh disables the button without changing its label", async ({ page }) => {
    await page.goto("/", { waitUntil: "domcontentloaded" });
    const refresh = page.getByRole("button", { name: "Refresh servers", exact: true });
    await expect(refresh).toBeEnabled();
    let release!: () => void;
    const responseGate = new Promise<void>((resolve) => {
        release = resolve;
    });
    await page.route("**/guilds", async (route) => {
        await responseGate;
        await route.fulfill({ json: guilds });
    });
    await refresh.click();
    await expect(refresh).toBeDisabled();
    await expect(refresh).toHaveText("Refresh servers");
    release();
    await expect(refresh).toBeEnabled();
});

test("account menu supports keyboard, dismissal, retry and sign-out", async ({ page }) => {
    await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("heading", { name: "Night Shift", exact: true })).toBeVisible();
    const trigger = page.getByRole("button", { name: "Account menu" });
    await expect(trigger).toBeEnabled();
    await trigger.focus();
    await trigger.press("Enter");
    const menu = page.getByRole("menu", { name: "Account menu", exact: true });
    await expect(menu).toContainText("dan@example.com");
    await page.keyboard.press("Escape");
    await expect(menu).toBeHidden();
    await expect(trigger).toBeFocused();
    await trigger.click();
    await page.getByRole("heading", { name: "Night Shift", exact: true }).click();
    await expect(menu).toBeHidden();
    await page.route("**/api/auth/sign-out", (route) =>
        route.fulfill({ status: 500, json: { message: "Failed" } }),
    );
    await trigger.click();
    await page.getByRole("menuitem", { name: "Sign out", exact: true }).click();
    await expect(menu.getByRole("alert")).toHaveText("Sign out failed. Try again.");
    await page.unroute("**/api/auth/sign-out");
    await page.getByRole("menuitem", { name: "Sign out", exact: true }).click();
    await expect(page).toHaveURL("/login");
    await expect(page.getByRole("button", { name: "Account menu" })).toHaveCount(0);
});

test("mobile sidebar traps focus, switches guilds, closes and restores focus", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
    await expect(page.getByRole("heading", { name: "Night Shift", exact: true })).toBeVisible();
    const trigger = page.getByRole("button", { name: "Open navigation" });
    const dialog = page.getByRole("dialog", { name: "Dashboard navigation" });
    await trigger.click();
    await expect(dialog).toBeVisible();
    await dialog.getByRole("button", { name: "Account menu" }).click();
    await expect(page.getByRole("menu", { name: "Account menu", exact: true })).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(dialog.getByRole("button", { name: "Account menu" })).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(dialog.getByRole("button", { name: "Close navigation" })).toBeFocused();
    await dialog.getByRole("button", { name: "Select server" }).click();
    await page.getByRole("link", { name: "Yin Community", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Yin Community", exact: true })).toBeVisible();
    await expect(dialog).toBeHidden();
    await trigger.click();
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();
    await expect(trigger).toBeFocused();
    expect(await page.evaluate(() => document.body.scrollWidth <= innerWidth)).toBe(true);
});

test.describe("mobile bottom sheet", () => {
    test.use({ hasTouch: true, viewport: { width: 390, height: 844 } });

    test("opens from the bottom and follows a swipe down to dismiss", async ({ page, context }) => {
        await page.goto("/", { waitUntil: "domcontentloaded" });
        const trigger = page.getByRole("button", { name: "Open navigation" });
        const drawer = page.getByRole("dialog", { name: "Dashboard navigation" });
        await trigger.tap();
        await expect(drawer).toBeVisible();
        await expect(drawer).not.toHaveAttribute("data-transitioning", "");
        await expect(drawer).toHaveCSS("transform", "matrix(1, 0, 0, 1, 0, 0)");
        const bounds = (await drawer.boundingBox())!;
        expect(bounds.x).toBe(0);
        expect(bounds.width).toBe(390);
        expect(bounds.y).toBeGreaterThan(200);
        expect(bounds.y + bounds.height).toBeCloseTo(844, 0);

        const handle = (await page.getByTestId("navigation-drag-handle").boundingBox())!;
        const x = handle.x + handle.width / 2;
        const y = handle.y + handle.height / 2;
        const cdp = await context.newCDPSession(page);
        await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x, y }] });
        for (let step = 1; step <= 8; step++) {
            await cdp.send("Input.dispatchTouchEvent", {
                type: "touchMove",
                touchPoints: [{ x, y: y + (bounds.height * 0.8 * step) / 8 }],
            });
        }
        expect((await drawer.boundingBox())!.y).toBeGreaterThan(bounds.y + 100);
        await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
        await cdp.detach();
        await expect(drawer).toBeHidden();
        await expect(trigger).toBeFocused();
    });

    test("backdrop dismissal and reduced motion work", async ({ page }) => {
        await page.emulateMedia({ reducedMotion: "reduce" });
        await page.goto("/", { waitUntil: "domcontentloaded" });
        const trigger = page.getByRole("button", { name: "Open navigation" });
        const drawer = page.getByRole("dialog", { name: "Dashboard navigation" });
        await trigger.tap();
        await expect(drawer).toBeVisible();
        await expect(drawer).not.toHaveAttribute("data-transitioning", "");
        expect(
            await drawer.evaluate((node) => parseFloat(getComputedStyle(node).transitionDuration)),
        ).toBeLessThan(0.001);
        await page.touchscreen.tap(20, 100);
        await expect(drawer).toBeHidden();
        await expect(trigger).toBeFocused();
        await trigger.tap();
        await drawer.getByRole("button", { name: "Close navigation" }).tap();
        await expect(drawer).toBeHidden();
    });

    test("resizing to desktop closes the sheet", async ({ page }) => {
        await page.goto("/", { waitUntil: "domcontentloaded" });
        await page.getByRole("button", { name: "Open navigation" }).tap();
        const drawer = page.getByRole("dialog", { name: "Dashboard navigation" });
        await expect(drawer).toBeVisible();
        await page.setViewportSize({ width: 1440, height: 1000 });
        await expect(drawer).toBeHidden();
        await expect(page.getByRole("complementary", { name: "Dashboard sidebar" })).toBeVisible();
    });
});

for (const mobile of [false, true]) {
    test(`save toasts wait for the API and handle failures (${mobile ? "mobile" : "desktop"})`, async ({
        page,
    }) => {
        if (mobile) await page.setViewportSize({ width: 390, height: 844 });
        await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
        await page.getByRole("textbox", { name: /^Command prefix/ }).fill("?");
        const notifications = page.getByRole("region", { name: /^Notifications/ });
        const notice = notifications.getByRole("listitem");
        const save = page.getByRole("button", { name: "Save general", exact: true });
        let release!: () => void;
        const responseGate = new Promise<void>((resolve) => {
            release = resolve;
        });
        const savedSettings = { ...settings("103"), commandPrefix: "?", activePrefix: "?" };
        await page.route("**/guilds/103/settings/general", async (route) => {
            expect(route.request().method()).toBe("PUT");
            expect(route.request().postDataJSON()).toEqual({
                commandPrefix: "?",
                translationLanguage: null,
            });
            await responseGate;
            await route.fulfill({ json: savedSettings });
        });
        await save.click();
        await expect(save).toBeDisabled();
        await expect(save).toHaveText("Save general");
        await expect(notice).toHaveCount(0);
        release();
        await expect(notice).toContainText("General settings saved.");
        await expect(notice).toContainText("Night Shift");
        await expect(notice).toHaveAttribute("data-type", "success");
        await expect(notice).toBeInViewport();
        await expect(page.getByRole("main")).not.toContainText("General settings saved.");
        await expect(page.getByRole("textbox", { name: /^Command prefix/ })).toHaveValue("?");
        await page.route("**/guilds/103/settings/general", (route) =>
            route.fulfill({
                status: 403,
                body: "You need Manage Server or Administrator permission.",
            }),
        );
        await save.click();
        await expect(notice).toContainText("You need Manage Server");
        await expect(notice).toHaveAttribute("data-type", "error");
        await expect(notice).toHaveCount(1);
        await expect(save).toBeEnabled();
        await page.route("**/guilds/103/settings/general", (route) =>
            route.fulfill({ json: savedSettings }),
        );
        await save.click();
        await expect(notice).toContainText("General settings saved.");
        await expect(notice).toHaveAttribute("data-type", "success");
        await notice.getByRole("button", { name: "Dismiss notification" }).click();
        await expect(notice).toHaveCount(0);
        expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(
            true,
        );
    });
}

test("invalid form input shows an error on the field without calling the API", async ({ page }) => {
    let requests = 0;
    await page.route("**/guilds/103/settings/general", (route) => {
        requests += 1;
        return route.fulfill({ json: settings("103") });
    });
    await page.route("**/guilds/103/ladder-rules", (route) => {
        requests += 1;
        return route.fulfill({ json: settings("103") });
    });
    await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });

    const prefix = page.getByRole("textbox", { name: /^Command prefix/ });
    await prefix.fill("a b");
    await page.getByRole("button", { name: "Save general", exact: true }).click();
    await expect(prefix).toHaveAttribute("aria-invalid", "true");
    await expect(prefix).toHaveAccessibleDescription(/contain no whitespace/);
    await prefix.fill("?");
    await expect(prefix).toHaveAttribute("aria-invalid", "false");

    const name = page.getByRole("textbox", { name: /^Command name/ });
    const response = page.getByRole("textbox", { name: /^Response text/ });
    await page.getByRole("button", { name: "Create or update command", exact: true }).click();
    await expect(name).toHaveAccessibleDescription(/1-32 characters/);
    await expect(response).toHaveAccessibleDescription(/1-2,000 characters/);

    await page
        .getByRole("navigation", { name: "Dashboard navigation", exact: true })
        .getByRole("link", { name: "Moderation", exact: true })
        .click();
    const timeoutDays = page.getByRole("spinbutton", { name: "Timeout days" });
    await timeoutDays.fill("29");
    await page.getByRole("button", { name: "Add ladder rule", exact: true }).click();
    await expect(timeoutDays).toHaveAccessibleDescription("Timeout must be 1-28 days.");
    expect(requests).toBe(0);
});

test("all settings actions report success in toasts", async ({ page }) => {
    await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
    const cases = [
        {
            section: "Social embeds",
            button: "Save social embeds",
            endpoint: "settings/social",
            method: "PUT",
            message: "Social embed settings saved.",
        },
        {
            section: "Custom commands",
            button: "Create or update command",
            endpoint: "custom-commands",
            method: "POST",
            message: "Custom command saved.",
        },
        {
            section: "Custom commands",
            button: "Remove",
            endpoint: "custom-commands/rules",
            method: "DELETE",
            message: "Custom command removed.",
        },
        {
            section: "Punishment ladder",
            button: "Add ladder rule",
            endpoint: "ladder-rules",
            method: "POST",
            message: "Punishment ladder rule added.",
        },
        {
            section: "Punishment ladder",
            button: "Remove",
            endpoint: "ladder-rules/1",
            method: "DELETE",
            message: "Punishment ladder rule removed.",
        },
    ];
    await page.getByRole("textbox", { name: /^Command name/ }).fill("rules");
    await page.getByRole("textbox", { name: /^Response text/ }).fill("Read #rules.");
    for (const action of cases) {
        if (action.section === "Punishment ladder" && !page.url().endsWith("/moderation")) {
            await page
                .getByRole("navigation", { name: "Dashboard navigation", exact: true })
                .getByRole("link", { name: "Moderation", exact: true })
                .click();
        }
        await page.route(`**/guilds/103/${action.endpoint}`, (route) => {
            expect(route.request().method()).toBe(action.method);
            return route.fulfill({ json: settings("103") });
        });
        await page
            .getByRole("region", { name: action.section, exact: true })
            .getByRole("button", { name: action.button, exact: true })
            .click();
        const notice = page.getByRole("region", { name: /^Notifications/ }).getByRole("listitem");
        await expect(notice).toContainText(action.message);
        await expect(notice).toBeInViewport();
        await notice.getByRole("button", { name: "Dismiss notification" }).click();
        await expect(notice).toHaveCount(0);
    }
});

test("a save finishing after a guild switch updates only its original guild", async ({ page }) => {
    await page.goto("/guilds/103", { waitUntil: "domcontentloaded" });
    await page.getByRole("textbox", { name: /^Command prefix/ }).fill("?");
    let release!: () => void;
    const responseGate = new Promise<void>((resolve) => {
        release = resolve;
    });
    await page.route("**/guilds/103/settings/general", async (route) => {
        await responseGate;
        await route.fulfill({
            json: { ...settings("103"), commandPrefix: "?", activePrefix: "?" },
        });
    });
    await page.getByRole("button", { name: "Save general", exact: true }).click();
    await expect(page.getByRole("button", { name: "Save general", exact: true })).toBeDisabled();
    await page.getByRole("button", { name: "Select server" }).click();
    await page.getByRole("link", { name: "Yin Community", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Yin Community", exact: true })).toBeVisible();
    const saved = page.waitForResponse("**/guilds/103/settings/general");
    release();
    await saved;
    await expect(page.getByRole("textbox", { name: /^Command prefix/ })).toBeEmpty();
    const notice = page.getByRole("region", { name: /^Notifications/ }).getByRole("listitem");
    await expect(notice).toContainText("General settings saved.");
    await expect(notice).toContainText("Night Shift");
    await expect(notice).not.toContainText("Yin Community");
    await page.getByRole("button", { name: "Select server" }).click();
    await page.getByRole("link", { name: "Night Shift", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Night Shift", exact: true })).toBeVisible();
    await expect(page.getByRole("textbox", { name: /^Command prefix/ })).toHaveValue("?");
});

test("guild list errors can be retried from the selector", async ({ page, context }) => {
    await context.addCookies([
        { name: "fixture-guilds", value: "error", url: "http://127.0.0.1:3010" },
    ]);
    await page.route("**/guilds", (route) =>
        route.fulfill({ status: 403, body: "Sign in again." }),
    );
    await page.goto("/", { waitUntil: "domcontentloaded" });
    await page.getByRole("button", { name: "Select server" }).click();
    const menu = page.getByRole("dialog", { name: "Select server", exact: true });
    await expect(menu.getByRole("alert")).toBeVisible({ timeout: 15_000 });
    await page.unroute("**/guilds");
    await menu.getByRole("button", { name: "Try again" }).click();
    await expect(menu.getByRole("link", { name: "Night Shift", exact: true })).toBeVisible();
});

test("the layout protects direct dashboard URLs but is absent from sign-in", async ({
    page,
    context,
}) => {
    await context.clearCookies();
    for (const path of ["/", "/guilds/103", "/guilds/103/moderation"]) {
        await page.goto(path, { waitUntil: "domcontentloaded" });
        await expect(page).toHaveURL("/login");
        await expect(page.getByRole("heading", { name: "Sign in to Yin" })).toBeVisible();
        await expect(page.getByRole("button", { name: "Select server" })).toHaveCount(0);
    }
});

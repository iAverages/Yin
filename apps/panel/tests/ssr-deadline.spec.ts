import { expect, test, type BrowserContext, type APIRequestContext } from "@playwright/test";

const configure = async (context: BrowserContext, cookies: Record<string, string> = {}) => {
    const id = crypto.randomUUID();
    await context.addCookies(
        Object.entries({
            "fixture-session": "1",
            "fixture-request-id": id,
            ...cookies,
        }).map(([name, value]) => ({ name, value, url: "http://127.0.0.1:3010" })),
    );
    return id;
};

const expectSingleFetches = async (request: APIRequestContext, id: string, paths: string[]) => {
    const response = await request.get(`http://127.0.0.1:33011/__requests/${id}`);
    const observation = await response.json();
    expect(observation.paths.sort()).toEqual([...paths].sort());
    expect(observation.aborted).toBe(0);
};

for (const route of [
    { path: "/", loading: "Loading servers", heading: "Servers" },
    { path: "/guilds/103", loading: "Loading server settings", heading: "Night Shift" },
    { path: "/guilds/103/moderation", loading: "Loading moderation", heading: "Night Shift" },
]) {
    test(`SSR deadline streams pending content without duplicate fetches: ${route.path}`, async ({
        page,
        context,
        request,
    }) => {
        const id = await configure(context, { "fixture-api-delay": "4500" });
        const apiRequests: string[] = [];
        const errors: string[] = [];
        page.on("request", (request) => {
            if (request.url().includes(":33011/")) apiRequests.push(request.url());
        });
        page.on("pageerror", (error) => errors.push(error.message));

        const response = await page.goto(route.path, { waitUntil: "commit" });
        expect(response?.headers()["cache-control"]).toContain("no-store");
        const main = page.getByRole("main");
        const loading = main.getByRole("status", { name: route.loading, exact: true });
        await expect(loading).toBeVisible();
        await expect(loading).toHaveAttribute("aria-busy", "true");
        await expect(
            page
                .getByRole("complementary", { name: "Dashboard sidebar" })
                .getByRole("link", { name: "Servers", exact: true }),
        ).toBeVisible();
        await expect(main.getByRole("alert")).toHaveCount(0);

        await expect(main.getByRole("heading", { name: route.heading, exact: true })).toBeVisible();
        await expect(page.getByRole("button", { name: "Select server" })).toBeEnabled();
        await expect(page.getByRole("status")).toHaveCount(0);
        expect(apiRequests).toEqual([]);
        expect(errors).toEqual([]);
        await expectSingleFetches(
            request,
            id,
            route.path === "/" ? ["/guilds"] : ["/guilds", "/guilds/103/settings"],
        );
    });
}

for (const slowQuery of ["guilds", "settings"]) {
    test(`SSR deadline preserves ready content while ${slowQuery} is pending`, async ({
        page,
        context,
        request,
    }) => {
        const id = await configure(context, { [`fixture-${slowQuery}-delay`]: "4500" });
        await page.goto("/guilds/103/moderation", { waitUntil: "commit" });
        const sidebar = page.getByRole("complementary", { name: "Dashboard sidebar" });
        const main = page.getByRole("main");
        if (slowQuery === "settings") {
            await expect(sidebar.getByRole("button", { name: "Select server" })).toContainText(
                "Night Shift",
            );
            await expect(main.getByRole("status", { name: "Loading moderation" })).toBeVisible();
        } else {
            await expect(sidebar.getByRole("status", { name: "Loading servers" })).toBeVisible();
            await expect(main.getByRole("region", { name: "Punishment ladder" })).toBeVisible();
        }
        await expect(main.getByRole("region", { name: "Punishment ladder" })).toBeVisible();
        await expect(sidebar.getByRole("button", { name: "Select server" })).toBeEnabled();
        await expect(page.getByRole("status")).toHaveCount(0);
        await expectSingleFetches(request, id, ["/guilds", "/guilds/103/settings"]);
    });
}

test("a failure after the SSR deadline replaces the skeleton and can be retried", async ({
    page,
    context,
    request,
}) => {
    await configure(context, { "fixture-settings-delay": "4500", "fixture-settings": "error" });
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.route("**/guilds/103/settings", (route) =>
        route.fulfill({
            status: 403,
            body: "You need Manage Server or Administrator permission.",
        }),
    );
    await page.goto("/guilds/103/moderation", { waitUntil: "commit" });
    const main = page.getByRole("main");
    const loading = main.getByRole("status", { name: "Loading moderation" });
    await expect(loading).toBeVisible();
    await expect(main.getByRole("alert")).toHaveCount(0);
    const alert = main.getByRole("alert");
    await expect(alert).toContainText("You need Manage Server");
    await expect(loading).toBeHidden();
    await expect(page.getByRole("button", { name: "Account menu" })).toBeEnabled();

    const recovered = await request.get("http://127.0.0.1:33011/guilds/103/settings", {
        headers: { cookie: "fixture-session=1" },
    });
    const settings = await recovered.json();
    await page.route("**/guilds/103/settings", (route) => route.fulfill({ json: settings }));
    await alert.getByRole("button", { name: "Try again" }).click();
    await expect(main.getByRole("region", { name: "Punishment ladder" })).toBeVisible();
    await expect(alert).toBeHidden();
    // The library may report the deliberately rejected SSR promise before hydration.
    expect(
        errors.filter((error) => error !== "You need Manage Server or Administrator permission."),
    ).toEqual([]);
});

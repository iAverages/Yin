import { expect, test } from "@playwright/test";

for (const target of ["guild", "user"]) {
    test(`${target} install success is public and returning to the panel still requires sign-in`, async ({
        page,
    }) => {
        await page.goto(`/install/discord/success?target=${target}`);

        await expect(
            page.getByRole("heading", { name: "Discord authorization complete" }),
        ).toBeVisible();
        await expect(page).toHaveURL(`/install/discord/success?target=${target}`);
        await expect(
            page.getByText(
                target === "user"
                    ? /You can now use Yin's commands in Discord/
                    : /Return to the panel to manage your server settings/,
            ),
        ).toBeVisible();

        await page.getByRole("link", { name: "Go to panel" }).click();
        await expect(page).toHaveURL("/login");
        await expect(page.getByRole("heading", { name: "Sign in to Yin" })).toBeVisible();
    });

    test(`${target} install failure can restart authorization without a session`, async ({
        page,
    }) => {
        const retryUrl = `http://127.0.0.1:33010/install/discord/${target}`;
        await page.route(retryUrl, (route) =>
            route.fulfill({ contentType: "text/plain", body: "Install restarted" }),
        );
        await page.goto(
            `/install/discord/error?target=${target}&error=access_denied&error_description=private-provider-details`,
        );

        await expect(
            page.getByRole("heading", { name: "Discord authorization failed" }),
        ).toBeVisible();
        await expect(page.getByRole("main")).not.toContainText("private-provider-details");
        await page.getByRole("link", { name: "Try again with Discord" }).click();
        await expect(page).toHaveURL(retryUrl);
        await expect(page.getByText("Install restarted")).toBeVisible();
    });
}

test("missing or invalid install targets fall back to a guild install", async ({ page }) => {
    for (const query of ["", "?target=https%3A%2F%2Fexample.com"]) {
        await page.goto(`/install/discord/error${query}`);
        await expect(
            page.getByRole("heading", { name: "Discord authorization failed" }),
        ).toBeVisible();
        await expect(page.getByRole("link", { name: "Try again with Discord" })).toHaveAttribute(
            "href",
            "http://127.0.0.1:33010/install/discord/guild",
        );
    }
});

test.describe("install results without JavaScript", () => {
    test.use({ javaScriptEnabled: false });

    for (const outcome of ["success", "error"]) {
        test(`${outcome} is server-rendered without a session`, async ({ page }) => {
            const response = await page.goto(`/install/discord/${outcome}?target=user`);

            expect(response?.status()).toBe(200);
            await expect(
                page.getByRole("heading", {
                    name: `Discord authorization ${outcome === "success" ? "complete" : "failed"}`,
                }),
            ).toBeVisible();
            await expect(page.getByRole("link", { name: "Go to panel" })).toHaveAttribute(
                "href",
                "/",
            );
            if (outcome === "error") {
                await expect(
                    page.getByRole("link", { name: "Try again with Discord" }),
                ).toHaveAttribute("href", "http://127.0.0.1:33010/install/discord/user");
            }
        });
    }
});

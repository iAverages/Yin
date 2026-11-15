import { betterAuth } from "better-auth";
import { memoryAdapter } from "better-auth/adapters/memory";
import { Hono } from "hono";
import { assert, test } from "vitest";

import { registerDiscordTokenRoute } from "./discord-token";

test("serves a stored Discord token to its signed-in owner only", async () => {
  const db: Record<string, Record<string, unknown>[]> = {
    user: [],
    session: [],
    account: [],
    verification: [],
  };
  const auth = betterAuth({
    baseURL: "http://auth.example.test",
    secret: "test-secret-8e82c999fa4e7e81ed71-a91f",
    database: memoryAdapter(db),
    emailAndPassword: { enabled: true },
    socialProviders: { discord: { clientId: "client-id", clientSecret: "client-secret" } },
    account: { encryptOAuthTokens: true },
  });
  const app = new Hono();
  registerDiscordTokenRoute(app, auth.api.getAccessToken);

  const signUp = await auth.handler(
    new Request("http://auth.example.test/api/auth/sign-up/email", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        name: "Fixture",
        email: "fixture@example.com",
        password: "fixture-password",
      }),
    }),
  );
  assert.equal(signUp.status, 200);
  const { user } = (await signUp.json()) as { user: { id: string } };
  // Stored before encryption was turned on, so it is still plaintext.
  db.account.push({
    id: "discord-account",
    userId: user.id,
    providerId: "discord",
    accountId: "42",
    accessToken: "discord-access-token",
    accessTokenExpiresAt: new Date(Date.now() + 60 * 60 * 1000),
    createdAt: new Date(),
    updatedAt: new Date(),
  });
  const cookie = signUp.headers
    .getSetCookie()
    .map((value) => value.split(";")[0])
    .join("; ");

  const signedIn = await app.request("/discord/access-token", { headers: { cookie } });
  assert.equal(signedIn.status, 200, await signedIn.clone().text());
  assert.equal(
    ((await signedIn.json()) as { accessToken: string }).accessToken,
    "discord-access-token",
  );

  const anonymous = await app.request("/discord/access-token");
  assert.equal(anonymous.status, 401);
});

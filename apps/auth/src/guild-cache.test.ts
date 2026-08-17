import { betterAuth } from "better-auth";
import { memoryAdapter } from "better-auth/adapters/memory";
import { test, vi, assert } from "vitest";

import { guildCacheWarmHook } from "./guild-cache";

const createAuth = (fetcher: typeof fetch, baseURL = "http://auth.example.test") =>
  betterAuth({
    baseURL,
    secret: "test-secret-8e82c999fa4e7e81ed71-a91f",
    database: memoryAdapter({ user: [], session: [], account: [], verification: [] }),
    emailAndPassword: { enabled: true },
    databaseHooks: {
      session: { create: { after: guildCacheWarmHook("http://api:3000", fetcher) } },
    },
  });

const signUp = async (auth: ReturnType<typeof createAuth>) => {
  const response = await auth.handler(
    new Request(`${auth.options.baseURL}/api/auth/sign-up/email`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        name: "Fixture",
        email: "fixture@example.com",
        password: "fixture-password",
      }),
    }),
  );
  assert.equal(response.status, 200);
  assert.ok(response.headers.get("set-cookie"));
  return response.json() as Promise<{ user: { id: string } }>;
};

for (const protocol of ["http", "https"]) {
  test(`${protocol} session creation warms the API using the newly persisted session`, async () => {
    const authenticatedUsers: string[] = [];
    const requests: Request[] = [];
    const fetcher: typeof fetch = async (input, init) => {
      const request = new Request(input, init);
      requests.push(request);
      // Exercise real Better Auth verification, just as the API's auth client does.
      const response = await auth.handler(
        new Request(`${auth.options.baseURL}/api/auth/get-session`, {
          headers: request.headers,
        }),
      );
      const session = (await response.json()) as { user: { id: string } } | null;
      if (!session) return new Response(null, { status: 401 });
      authenticatedUsers.push(session.user.id);
      return Response.json([]);
    };
    const auth = createAuth(fetcher, `${protocol}://auth.example.test`);
    const result = await signUp(auth);

    assert.deepEqual(authenticatedUsers, [result.user.id]);
    assert.equal(requests.length, 1); // Session validation must not re-enter the create hook.
    assert.equal(requests[0].url, "http://api:3000/guilds");
    assert.equal(requests[0].method, "GET");
    assert.equal(requests[0].redirect, "error"); // Never forward a session cookie to a redirect target.
    assert.ok(
      requests[0].headers
        .get("cookie")
        ?.startsWith(
          protocol === "https"
            ? "__Secure-better-auth.session_token="
            : "better-auth.session_token=",
        ),
    );
  });
}

for (const status of [401, 503]) {
  test(`an API ${status} does not fail sign-in or log its response body`, async () => {
    const warnings = vi.spyOn(console, "warn").mockImplementation(() => {});
    const auth = createAuth(async () => new Response("private upstream details", { status }));

    await signUp(auth);

    assert.deepEqual(warnings.mock.calls, [
      ["Guild cache warm-up failed; it will retry on the next guild request."],
    ]);
  });
}

test("an unreachable API does not fail sign-in or log request credentials", async () => {
  const warnings = vi.spyOn(console, "warn").mockImplementation(() => {});
  const auth = createAuth(async () => {
    throw new Error("private request credentials");
  });

  await signUp(auth);

  assert.deepEqual(warnings.mock.calls, [
    ["Guild cache warm-up failed; it will retry on the next guild request."],
  ]);
});

test("warm-up passes an eight-second deadline to fetch and tolerates its expiry", async () => {
  vi.spyOn(console, "warn").mockImplementation(() => {});
  const deadlines: number[] = [];
  vi.spyOn(AbortSignal, "timeout").mockImplementation((milliseconds) => {
    deadlines.push(milliseconds);
    return AbortSignal.abort(new DOMException("Timed out", "TimeoutError"));
  });
  let aborted = false;
  const auth = createAuth(async (_input, init) => {
    aborted = init?.signal?.aborted === true;
    init?.signal?.throwIfAborted();
    return Response.json([]);
  });

  await signUp(auth);

  assert.deepEqual(deadlines, [8_000]);
  assert.equal(aborted, true);
});

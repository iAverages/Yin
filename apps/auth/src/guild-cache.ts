import type { GenericEndpointContext } from "better-auth";
import { makeSignature } from "better-auth/crypto";

export const guildCacheWarmHook =
  (apiServiceUrl: string, fetcher = fetch) =>
  async (session: { token: string }, context: GenericEndpointContext | null) => {
    if (!context) return;

    try {
      // The new session is persisted, but its browser cookie has not been sent yet.
      const signature = await makeSignature(session.token, context.context.secret);
      const cookie = `${context.context.authCookies.sessionToken.name}=${encodeURIComponent(`${session.token}.${signature}`)}`;
      const response = await fetcher(new URL("/guilds", apiServiceUrl), {
        headers: { cookie },
        redirect: "error",
        signal: AbortSignal.timeout(8_000),
      });
      await response.body?.cancel();
      if (!response.ok) throw new Error("Guild cache warm-up request failed");
    } catch {
      // dont expose session credentials or upstream payloads
      console.warn("Guild cache warm-up failed; it will retry on the next guild request.");
    }
  };

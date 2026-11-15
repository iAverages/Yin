import type { Hono } from "hono";

import type { auth } from "./auth";

export const registerDiscordTokenRoute = (
  app: Hono,
  getAccessToken: typeof auth.api.getAccessToken,
) => {
  app.get("/discord/access-token", (c) =>
    getAccessToken({
      body: { providerId: "discord" },
      headers: c.req.raw.headers,
      asResponse: true,
    }),
  );
};

import type { Hono } from "hono";

import { auth } from "./auth";
import type { AuthConfig } from "./config";

type InstallTarget = "guild" | "user";

const guildInstallScopes = ["guilds", "bot", "applications.commands"];
const userInstallScopes = ["applications.commands"];

type SocialSignInResponse = {
  redirect: boolean;
  url?: string;
};

export function registerInstallRoutes(app: Hono, config: AuthConfig) {
  app.get("/install/discord/guild", async (c) => {
    return startDiscordInstall(c.req.raw, config, "guild");
  });

  app.get("/install/discord/user", async (c) => {
    return startDiscordInstall(c.req.raw, config, "user");
  });

  app.get("/install/discord/success", (c) => {
    return c.text("Discord authorization complete. You can close this tab.");
  });

  app.get("/install/discord/error", (c) => {
    return c.text("Discord authorization failed.", 400);
  });
}

async function startDiscordInstall(
  request: Request,
  config: AuthConfig,
  target: InstallTarget,
) {
  const scopes = target === "guild" ? guildInstallScopes : userInstallScopes;
  const response = await auth.handler(
    new Request(`${config.betterAuthUrl}/api/auth/sign-in/social`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        origin: new URL(request.url).origin,
      },
      body: JSON.stringify({
        provider: "discord",
        disableRedirect: true,
        requestSignUp: true,
        callbackURL: withTarget(config.installSuccessUrl, target),
        errorCallbackURL: withTarget(config.installErrorUrl, target),
        scopes,
        additionalData: { installTarget: target },
      }),
    }),
  );

  if (!response.ok) {
    return response;
  }

  const body = (await response.json()) as SocialSignInResponse;
  if (!body.url) {
    return new Response("missing Discord authorization URL", { status: 502 });
  }

  const redirectUrl = new URL(body.url);
  redirectUrl.searchParams.set("integration_type", target === "guild" ? "0" : "1");

  const redirect = new Response(null, {
    status: 302,
    headers: { location: redirectUrl.toString() },
  });
  copySetCookieHeaders(response, redirect);
  return redirect;
}

function withTarget(url: string, target: InstallTarget) {
  const result = new URL(url);
  result.searchParams.set("target", target);
  return result.toString();
}

function copySetCookieHeaders(from: Response, to: Response) {
  const headers = from.headers as Headers & { getSetCookie?: () => string[] };
  const setCookies = headers.getSetCookie?.() ?? [];

  if (setCookies.length > 0) {
    for (const cookie of setCookies) {
      to.headers.append("set-cookie", cookie);
    }
    return;
  }

  const cookie = from.headers.get("set-cookie");
  if (cookie) {
    to.headers.append("set-cookie", cookie);
  }
}

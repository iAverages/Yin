import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { cors } from "hono/cors";

import { auth } from "./auth";
import { loadConfig } from "./config";
import { registerInstallRoutes } from "./install";

const config = loadConfig();
const app = new Hono();

registerInstallRoutes(app, config);

app.use(
  "/api/auth/*",
  cors({
    origin: config.trustedOrigins,
    allowHeaders: ["Content-Type", "Authorization"],
    allowMethods: ["GET", "POST", "OPTIONS"],
    exposeHeaders: ["Content-Length"],
    credentials: true,
    maxAge: 600,
  }),
);

app.on(["GET", "POST"], "/api/auth/*", (c) => auth.handler(c.req.raw));

serve(
  {
    fetch: app.fetch,
    hostname: config.host,
    port: config.port,
  },
  (info) => {
    console.log(`auth api listening on http://${info.address}:${info.port}`);
  },
);

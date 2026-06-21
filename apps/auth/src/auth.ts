import { betterAuth } from "better-auth";

import { loadConfig } from "./config";
import { createDatabasePool } from "./database";

const config = loadConfig();

export const auth = betterAuth({
  database: createDatabasePool(config.databaseUrl),
  secret: config.betterAuthSecret,
  baseURL: config.betterAuthUrl,
  trustedOrigins: config.trustedOrigins,
  advanced: {
    crossSubDomainCookies: {
      enabled: true,
    },
  },
  socialProviders: {
    discord: {
      clientId: config.discordClientId,
      clientSecret: config.discordClientSecret,
      permissions: config.discordBotPermissions,
      prompt: "consent",
    },
  },
});

import { betterAuth } from "better-auth";

import { loadConfig } from "./config";
import { createDatabasePool } from "./database";
import { guildCacheWarmHook } from "./guild-cache";

const config = loadConfig();
const database = createDatabasePool(config.databaseUrl);

export const auth = betterAuth({
  database,
  databaseHooks: {
    session: {
      create: {
        after: guildCacheWarmHook(config.apiServiceUrl),
      },
    },
  },
  secret: config.betterAuthSecret,
  baseURL: config.betterAuthUrl,
  trustedOrigins: config.trustedOrigins,
  advanced: {
    crossSubDomainCookies: {
      enabled: true,
      domain: config.cookieDomain,
    },
  },
  socialProviders: {
    discord: {
      clientId: config.discordClientId,
      clientSecret: config.discordClientSecret,
      permissions: config.discordBotPermissions,
      scope: ["guilds"],
      prompt: "consent",
    },
  },
});

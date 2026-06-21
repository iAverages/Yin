import { createEnv } from "@t3-oss/env-core";
import { z } from "zod";

export type AuthConfig = {
  host: string;
  port: number;
  betterAuthUrl: string;
  betterAuthSecret: string;
  databaseUrl: string;
  discordClientId: string;
  discordClientSecret: string;
  discordBotPermissions: number;
  installSuccessUrl: string;
  installErrorUrl: string;
  trustedOrigins: string[];
};

const env = createEnv({
  server: {
    AUTH_HOST: z.string().min(1).default("0.0.0.0"),
    AUTH_PORT: z.coerce.number().int().min(1).max(65535).default(3001),
    BETTER_AUTH_URL: z.url(),
    BETTER_AUTH_SECRET: z.string().min(32),
    DATABASE_URL: z.url(),
    DISCORD_CLIENT_ID: z.string().min(1),
    DISCORD_CLIENT_SECRET: z.string().min(1),
    DISCORD_BOT_PERMISSIONS: z.coerce.number().int().min(0).default(0),
    AUTH_INSTALL_SUCCESS_URL: z.url().optional(),
    AUTH_INSTALL_ERROR_URL: z.url().optional(),
    AUTH_TRUSTED_ORIGINS: z
      .string()
      .default("http://api.yin.localhost:3000,http://auth.yin.localhost:3001")
      .transform((value) =>
        value
          .split(",")
          .map((item) => item.trim())
          .filter(Boolean),
      )
      .pipe(z.array(z.url()).min(1)),
  },
  runtimeEnv: process.env,
  emptyStringAsUndefined: true,
});

export function loadConfig(): AuthConfig {
  return {
    host: env.AUTH_HOST,
    port: env.AUTH_PORT,
    betterAuthUrl: env.BETTER_AUTH_URL,
    betterAuthSecret: env.BETTER_AUTH_SECRET,
    databaseUrl: env.DATABASE_URL,
    discordClientId: env.DISCORD_CLIENT_ID,
    discordClientSecret: env.DISCORD_CLIENT_SECRET,
    discordBotPermissions: env.DISCORD_BOT_PERMISSIONS,
    installSuccessUrl:
      env.AUTH_INSTALL_SUCCESS_URL ??
      `${env.BETTER_AUTH_URL}/install/discord/success`,
    installErrorUrl:
      env.AUTH_INSTALL_ERROR_URL ?? `${env.BETTER_AUTH_URL}/install/discord/error`,
    trustedOrigins: env.AUTH_TRUSTED_ORIGINS,
  };
}

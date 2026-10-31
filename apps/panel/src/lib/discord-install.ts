import { z } from "zod";

import { env } from "../env";

export const discordInstallSearch = z.object({
    target: z.enum(["guild", "user"]).catch("guild"),
});

export const discordInstallUrl = (target: "guild" | "user") =>
    new URL(`/install/discord/${target}`, env.VITE_AUTH_URL).toString();

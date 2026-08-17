import { z } from "zod";

export const discordInstallSearch = z.object({
    target: z.enum(["guild", "user"]).catch("guild"),
});

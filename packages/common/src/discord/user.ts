import { z } from "zod";

export const userSchema = z.object({
    id: z.string(),
    username: z.string(),
    discriminator: z.string().max(4),
    avatar: z.string().nullable(),
    bot: z.boolean().optional(),
    system: z.boolean().optional(),
    mfa_enabled: z.boolean().optional(),
    banner: z.string().nullish(),
    accent_colour: z.number().nullish(),
    locale: z.string().optional(),
    verified: z.boolean().optional(),
    email: z.string().nullish(),
    flags: z.number().optional(),
    premium_type: z.number().optional(),
    public_flags: z.number().optional(),
});

export type User = z.infer<typeof userSchema>;

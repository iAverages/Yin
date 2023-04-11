import { req } from "../manager";
import { Routes } from "../routes";
import { z } from "zod";
import { guild } from "./index";

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

export const services = [
    "battlenet",
    "ebay",
    "epicgames",
    "facebook",
    "github",
    "leagueoflegends",
    "paypal",
    "playstation",
    "reddit",
    "riotgames",
    "spotify",
    "skype",
    "steam",
    "tiktok",
    "twitch",
    "twitter",
    "xbox",
    "youtube",
] as const;

export const connectionSchema = z.object({
    id: z.string(),
    name: z.string(),
    type: z.string(),
    revoked: z.boolean().optional(),
    // integrations: z.array(integrationSchema).optional(),
    verified: z.boolean(),
    friend_sync: z.boolean(),
    show_activity: z.boolean(),
    two_way_link: z.boolean(),
    visibility: z.number(),
});

export type GetUserUrlParts = {
    "user.id": string;
};

export const handler = {
    getCurrentUser: () => {
        return req({ method: "GET", schema: userSchema, url: Routes.USERS_ME, urlParts: null });
    },
    getUser: (parts: GetUserUrlParts) => {
        return req({ method: "GET", schema: userSchema, url: Routes.USER, urlParts: parts });
    },
    modifyCurrentUser: () => {
        return req({
            method: "PATCH",
            schema: z.object({ username: z.string() }),
            url: Routes.USERS_ME,
            urlParts: null,
        });
    },
    getCurrentUserGuilds: () => {
        return req({
            method: "GET",
            schema: guild.guildPartialSchema.array(),
            url: Routes.USERS_ME_GUILDS,
            urlParts: null,
        });
    },
    getCurrentUserGuildMember: ({ id }: { id: string }) => {},
};

import { z } from "zod";

import { req } from "../manager";
import { Routes } from "../routes";
import { guildUserSchema } from "./guild";
import { userSchema } from "./user";

export type InterationResponseUrlParts = {
    "interaction.id": string;
    "interaction.token": string;
};

export type InteractionResponseBody = {
    type: InteractionResponseType;
    data?: InteractionResponseData;
};

export enum InteractionType {
    "PING",
    "APPLICATION_COMMAND",
    "MESSAGE_COMPONENT",
    "APPLICATION_COMMAND_AUTOCOMPLETE",
    "MODAL_SUBMIT",
}

export enum InteractionResponseType {
    "PONG",
    "CHANNEL_MESSAGE_WITH_SOURCE",
    "DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE",
    "DEFERRED_UPDATE_MESSAGE",
    "UPDATE_MESSAGE",
    "APPLICATION_COMMAND_AUTOCOMPLETE_RESULT",
    "MODAL",
}

export const interactionSchema = z.object({
    id: z.string(),
    application_id: z.string(),
    type: z.nativeEnum(InteractionType),
    data: z.object({}).optional(), // TODO:
    guild_id: z.string().optional(),
    channel: z.object({}).optional(), // TODO: channel schema
    channel_id: z.string().optional(),
    member: guildUserSchema.optional(), // TODO: member schema
    user: userSchema.optional(),
    token: z.string(),
    version: z.number(),
    message: z.object({}).optional(), // TODO: message schema
    app_permissions: z.string().optional(),
    locale: z.string().optional(),
    guild_locale: z.string().optional(),
});

export type Interaction = z.infer<typeof interactionSchema>;

export const interactionResponseDataSchema = z.object({
    tts: z.boolean().optional(),
    content: z.string().optional(),
    embeds: z.array(z.object({})).optional(),
    allowed_mentions: z.object({}).optional(),
    flags: z.number().optional(),
    components: z.array(z.object({})).optional(),
    attachments: z.array(z.object({})).optional(),
});

export type InteractionResponseData = z.infer<typeof interactionResponseDataSchema>;

export const interactionResponseSchema = z.object({
    type: z.nativeEnum(InteractionResponseType),
    data: interactionResponseDataSchema.optional(),
});

export const interactionHandler = {
    respond: (body: InteractionResponseBody, parts: InterationResponseUrlParts) => {
        return req({
            method: "POST",
            schema: interactionResponseSchema,
            url: Routes.INTERACTION_CREATE,
            urlParts: parts,
            body,
        });
    },
};

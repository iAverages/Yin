import { z } from "zod";

import { req } from "../manager";
import { Routes } from "../routes";
import { embedSchema } from "../structs/channel";
import type { ExecuteWebhookBody } from "../structs/webhook";
import { guildUserSchema } from "./guild";
import { userSchema } from "./user";

export type InterationResponseUrlParts = {
    "interaction.id": string;
    "interaction.token": string;
};

export type InterationFollowupUrlParts = {
    "application.id": string;
    "interaction.token": string;
};

export type InteractionResponseBody = {
    type: InteractionResponseType;
    data?: InteractionResponseData;
};

// TODO: Complete type
export type InteractionEditResponseBody = {
    content?: string;
    embeds?: (typeof embedSchema)["_output"][];
    allowed_mentions?: unknown;
    components?: unknown[];
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

const _interactionOptionSchema = {
    name: z.string(),
    value: z.any().optional(),
    type: z.number(),
};
const _interactionOptionSchema1 = {
    ..._interactionOptionSchema,
    options: z.object(_interactionOptionSchema).array().optional(),
};
const _interactionOptionSchema2 = {
    ..._interactionOptionSchema,
    options: z.object(_interactionOptionSchema1).array().optional(),
};
const _interactionOptionSchema3 = {
    ..._interactionOptionSchema,
    options: z.object(_interactionOptionSchema2).array().optional(),
};
const _interactionOptionSchema4 = {
    ..._interactionOptionSchema,
    options: z.object(_interactionOptionSchema3).array().optional(),
};

export const interactionOptionSchema = z
    .object({
        ..._interactionOptionSchema,
        options: z.object(_interactionOptionSchema4).array().optional(),
    })
    .array();

export const interactionSchema = z.object({
    id: z.string(),
    application_id: z.string(),
    type: z.nativeEnum(InteractionType),
    data: z
        .object({
            id: z.string(),
            name: z.string(),
            type: z.number(),
            options: interactionOptionSchema.optional(),
            value: z.any().optional(),
        })
        .optional(), // TODO:
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
    embeds: z.array(embedSchema).optional(),
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
    followup: (body: ExecuteWebhookBody, parts: InterationFollowupUrlParts) => {
        return req({
            method: "POST",
            schema: null,
            url: Routes.INTERACTION_FOLLOWUP,
            urlParts: parts,
            body,
        });
    },
    edit: (body: InteractionEditResponseBody, parts: InterationFollowupUrlParts) => {
        return req({
            method: "PATCH",
            schema: null,
            url: Routes.INTERACTION_EDIT,
            urlParts: parts,
            body,
        });
    },
};

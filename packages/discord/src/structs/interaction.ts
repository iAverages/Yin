import { req } from "../manager";
import { Routes } from "../routes";
import { z } from "zod";

// const interactionType = z.enum([1, 2, 3, 4, 5]);

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
});

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

export const handler = {
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

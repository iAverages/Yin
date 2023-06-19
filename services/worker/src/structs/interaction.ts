import api, { interaction } from "@yin/discord";
import type { worker } from "@yin/grpc";

import { type InteractionHandler } from "~/interactions";

export const createInteraction = (interactionData: worker.InteractionRequest, _meta: InteractionHandler["meta"]) => {
    let hasReplied = false;
    const options = interaction.interactionOptionSchema.parse(JSON.parse(interactionData.data?.options ?? "[]"));

    return {
        id: interactionData.id,
        type: interactionData.type,
        token: interactionData.token,
        guildId: interactionData.guildId,
        channelId: interactionData.channelId,
        websocketInfo: interactionData.websocketInfo,
        applicationId: interactionData.applicationId,
        data: {
            id: interactionData.data?.id ?? "",
            name: interactionData.data?.name ?? "",
            type: interactionData.data?.type ?? 0,
            options: options[0] ?? [],
        },
        reply: (data: interaction.InteractionResponseData) => {
            if (hasReplied) {
                throw new Error("Already replied to interaction");
            }
            hasReplied = true;
            return api.interaction.respond(
                {
                    type: 4,
                    data,
                },
                {
                    "interaction.id": interactionData.id,
                    "interaction.token": interactionData.token,
                }
            );
        },
        edit: (data: interaction.InteractionEditResponseBody) => {
            if (!hasReplied) {
                throw new Error("Cannot edit interaction before replying");
            }
            return api.interaction.edit(data, {
                "application.id": interactionData.id,
                "interaction.token": interactionData.token,
            });
        },
    };
};

export type Interaction = ReturnType<typeof createInteraction>;

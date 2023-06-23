import api, { interaction } from "@yin/discord";
import type { worker } from "@yin/grpc";

import { type InteractionHandler } from "~/interactions";

export const createInteraction = (interactionData: worker.InteractionRequest, _meta: InteractionHandler["meta"]) => {
    if (!interactionData.data) {
        throw new Error("No data found in request");
    }

    // required message types are not yet supported in ts-proto
    if (!interactionData.gatewayMeta) {
        throw new Error("No gatewayMeta found in request");
    }

    let hasReplied = false;
    const options = interaction.interactionOptionSchema.parse(JSON.parse(interactionData.data?.options ?? "[]"));

    return {
        id: interactionData.id,
        type: interactionData.type,
        token: interactionData.token,
        guildId: interactionData.guildId,
        channelId: interactionData.channelId,
        applicationId: interactionData.applicationId,
        data: {
            id: interactionData.data?.id ?? "",
            name: interactionData.data?.name ?? "",
            type: interactionData.data?.type ?? 0,
            options: options[0] ?? [],
        },
        gatewayMeta: interactionData.gatewayMeta,
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

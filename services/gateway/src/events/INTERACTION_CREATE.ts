import { logger, trytm } from "@yin/common";
import api, { interaction } from "@yin/discord";

import { type Event } from "~/events";

export default async ({ packet, service, wsInfo }: Event<interaction.Interaction>) => {
    if (!packet.d) {
        return;
    }
    const [data, error] = await trytm(interaction.interactionSchema.parseAsync(packet.d));

    if (error) {
        logger.error({ error, packet: packet.d }, "Failed to parse interaction");
        if (!packet.d.id || !packet.d.token) {
            logger.warn("No id or token found, cannot response to interaction");
            return;
        }
        api.interaction.respond(
            {
                type: 4,
                data: {
                    content: "An error occurred.",
                },
            },
            { "interaction.id": packet.d.id, "interaction.token": packet.d.token }
        );
        return;
    }

    const guildId = data.guild_id;
    const userId = data.member?.user.id ?? data.user?.id;

    logger.debug(data.data?.options, "Handling interaction");

    service.services.worker.handleInteraction(
        {
            id: data.id,
            type: data.type,
            token: data.token,
            userId: data.member?.user.id ?? data.user?.id ?? "",
            guildId: data.guild_id,
            channelId: data.channel_id,

            gatewayMeta: {
                pod: service.env.K3S_POD_NAME,
                websocketMeta: {
                    ping: wsInfo.ping,
                },
            },
            applicationId: data.application_id,
            data: {
                id: data.data?.id ?? "",
                name: data.data?.name ?? "",
                type: data.data?.type ?? 0,
                options: JSON.stringify(data.data?.options ?? []),
            },
        },
        (err, res) => {
            if (err) {
                logger.error(err, "Failed to handle interaction");
                return;
            }
            logger.debug(res, "Handled interaction event");
        }
    );
    if (!userId || !guildId) {
        logger.warn("No user or guild id found");
        return;
    }
};

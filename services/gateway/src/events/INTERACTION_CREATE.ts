import { logger } from "@yin/common";
import { type interaction } from "@yin/discord";

import { type Event } from "~/events";

export default async ({ packet, service, wsInfo }: Event<interaction.Interaction>) => {
    if (!packet.d) {
        return;
    }

    const guildId = packet.d.guild_id;
    const userId = packet.d.member?.user.id ?? packet.d.user?.id;
    service.services.worker.handleInteraction(
        {
            id: packet.d.id,
            type: packet.d.type,
            token: packet.d.token,
            userId: packet.d.member?.user.id ?? packet.d.user?.id ?? "",
            guildId: packet.d.guild_id,
            channelId: packet.d.channel_id,
            websocketInfo: {
                ping: wsInfo.ping,
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

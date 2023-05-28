import { logger } from "@yin/common";
import { type interaction } from "@yin/discord";

import { DiscordPacket } from "~/packets/BasePacket";
import { ServiceMeta } from "~/service";

export default async (service: ServiceMeta, packet: DiscordPacket<interaction.Interaction>) => {
    if (!packet.d) {
        return;
    }
    console.log(packet);

    const guildId = packet.d.guild_id;
    const userId = packet.d.member?.user.id ?? packet.d.user?.id;

    if (!userId || !guildId) {
        logger.warn("No user or guild id found");
        return;
    }

    service.services.worker.handlePacket(
        {
            body: JSON.stringify(packet),
        },
        (err, b) => {
            if (err) {
                logger.error(err);
                return;
            }

            console.log(b.success ? "handled packet" : "failed to handle packet");
        }
    );
};

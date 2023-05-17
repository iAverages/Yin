import { logger } from "@yin/common";
import api, { type interaction } from "@yin/discord";
import { database } from "@yin/grpc";

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

    service.services.database.logEvent(
        {
            createdAt: new Date(),
            discordEvent: "INTERACTION_CREATE",
            discordGuildId: guildId,
            discordUserId: userId,
        },
        (err, b) => {
            if (err) {
                logger.error(err);
                return;
            }

            console.log(b.success ? "logged event" : "failed to log event");
        }
    );

    // service.services.worker.
    const res = await api.interaction.respond(
        {
            type: 4,
            data: {
                content: "Hello world!",
            },
        },
        {
            "interaction.id": packet.d.id,
            "interaction.token": packet.d.token,
        }
    );
};

import dns from "dns";

import { logger } from "@yin/common";
import api, { type interaction } from "@yin/discord";

import { DiscordPacket } from "~/packets/BasePacket";
import { ServiceMeta } from "~/service";

export default async (service: ServiceMeta, packet: DiscordPacket<interaction.Interaction>) => {
    if (!packet.d) {
        return;
    }
    console.log(packet);

    const guildId = packet.d.guild_id;
    const userId = packet.d.member?.user.id ?? packet.d.user?.id;
    const avatar = packet.d.member?.user.avatar ?? packet.d.user?.avatar;
    const name = packet.d.member?.user.username ?? packet.d.user?.username;

    if (!userId || !guildId) {
        logger.warn("No user or guild id found");
        return;
    }

    // service.services.database.addUser({ avatar: avatar ?? "", id: userId, name: name ?? "" }, (err, b) => {
    //     if (err) {
    //         logger.error(err);
    //         return;
    //     }

    //     console.log(b.success ? "added user" : "failed to add user");
    // });

    // service.services.database.logEvent(
    //     {
    //         createdAt: new Date(),
    //         discordEvent: "INTERACTION_CREATE",
    //         discordGuildId: guildId,
    //         discordUserId: userId,
    //     },
    //     (err, b) => {
    //         if (err) {
    //             logger.error(err);
    //             return;
    //         }

    //         console.log(b.success ? "logged event" : "failed to log event");
    //     }
    // );

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

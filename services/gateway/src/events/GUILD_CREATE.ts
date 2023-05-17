import { ZodError } from "zod";

import { logger } from "@yin/common";
import { guild } from "@yin/discord";
import { database } from "@yin/grpc";

import { ServiceMeta } from "~/service";
import { DiscordPacket } from "../packets/BasePacket";

export default async (service: ServiceMeta, packet: DiscordPacket<any>) => {
    if (!packet.d) {
        return;
    }

    try {
        const eventGuild = await guild.guildSchema.parseAsync(packet.d);

        service.services.database.addGuild(
            database.Guild.create({
                icon: eventGuild.icon,
                id: eventGuild.id,
                name: eventGuild.name,
            }),
            (err, b) => {
                if (err) {
                    logger.error(err);
                    return;
                }

                console.log(b.success ? "added guild" : "failed to add guild");
            }
        );
    } catch (err) {
        if (err instanceof ZodError) {
            console.log(err.message);
            return;
        }

        logger.error(err);
    }
};

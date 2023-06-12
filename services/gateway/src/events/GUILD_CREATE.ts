import { ZodError } from "zod";

import { logger } from "@yin/common";
import { guild } from "@yin/discord";

import { type Event } from "~/events";

export default async ({ service, packet }: Event) => {
    if (!packet.d) {
        logger.debug("GUILD_CREATE: no data");
        return;
    }

    try {
        const eventGuild = await guild.guildSchema.parseAsync(packet.d);
        logger.debug(
            {
                event: "GUILD_CREATE",
                data: packet.d,
            },
            "Got GUILD_CREATE event"
        );

        service.services.database.addGuild(
            {
                icon: eventGuild.icon,
                id: eventGuild.id,
                name: eventGuild.name,
            },
            (err, res) => {
                if (err) {
                    logger.error(err);
                    return;
                }
                logger.debug(res ? "Added guild" : "Failed to add guild");
            }
        );
    } catch (err) {
        if (err instanceof ZodError) {
            logger.error(
                {
                    event: "GUILD_CREATE",
                    data: packet.d,
                    error: err,
                },
                "Failed to parse GUILD_CREATE event"
            );
            return;
        }

        logger.error(err);
    }
};

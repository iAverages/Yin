import { logger } from "@yin/common";

import { type Event } from "~/events";
import GUILD_CREATE from "~/events/GUILD_CREATE";

export default async (event: Event) => {
    if (!event.packet.d) {
        logger.debug("GUILD_UPDATE: no data");
        return;
    }
    logger.debug("GUILD_UPDATE, calling GUILD_CREATE handler");
    GUILD_CREATE(event);
};

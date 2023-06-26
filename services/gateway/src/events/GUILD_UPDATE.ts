import { logger, metrics } from "@yin/common";

import { type Event } from "~/events";
import GUILD_CREATE from "~/events/GUILD_CREATE";

const counter = new metrics.Counter({
    help: "Number of GUIKD_UPDATE events received",
    name: "gateway_guild_update_events_received",
});

export default async (event: Event) => {
    if (!event.packet.d) {
        logger.debug("GUILD_UPDATE: no data");
        return;
    }
    counter.inc();
    logger.debug("GUILD_UPDATE, calling GUILD_CREATE handler");
    GUILD_CREATE(event);
};

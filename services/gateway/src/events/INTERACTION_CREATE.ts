import { logger } from "@yin/common";
import { type interaction } from "@yin/discord";

import { type Event } from "~/events";

export default async ({ packet }: Event<interaction.Interaction>) => {
    if (!packet.d) {
        return;
    }

    const guildId = packet.d.guild_id;
    const userId = packet.d.member?.user.id ?? packet.d.user?.id;

    if (!userId || !guildId) {
        logger.warn("No user or guild id found");
        return;
    }
};

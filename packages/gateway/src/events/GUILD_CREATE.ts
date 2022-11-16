import { GuildObject } from "@yin/common";
import { Client } from "../Client";
import { DiscordPacket } from "../packets/BasePacket";

export default (core: Client, packet: DiscordPacket<GuildObject>) => {
    if (!packet.d) {
        return;
    }
    // const guild = new Guild(packet.d);
    core.redis.publish("dwad", packet.d);
    // core.emit("guildCreate", guild);
};

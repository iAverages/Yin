import { DiscordEvents } from "@yin/common";
import { Client } from "../Client";
import { DiscordPacket } from "../packets/BasePacket";

export default (core: Client, packet: DiscordPacket) => {
    core.redis.publish(DiscordEvents.READY, packet);
};

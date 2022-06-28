import { MessagePacket } from "@yin/common";
import { Client } from "../Client";
import { DiscordPacket } from "../packets/BasePacket";

export default (core: Client, packet: DiscordPacket<MessagePacket>) => {
    if (!packet.d) {
        return;
    }
    // const message = new Message(packet.d);
    core.redis.instance.publish("1.0.0:messageCreate", JSON.stringify(packet.d));
};

import { DiscordEvents, MessagePacket } from "@yin/common";
import { Client } from "../Client";
import { DiscordPacket } from "../packets/BasePacket";

export default (core: Client, packet: DiscordPacket<MessagePacket>) => {
    if (!packet.d) {
        return;
    }
    core.work(DiscordEvents.MESSAGE_CREATE, packet.d);
};

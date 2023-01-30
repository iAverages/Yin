import { DiscordPacket } from "../packets/BasePacket";

export default (packet: DiscordPacket<any>) => {
    if (!packet.d) {
        return;
    }
    console.log("MESSAGE Create packet received");
};

import { ServiceMeta } from "~/service";
import { DiscordPacket } from "../packets/BasePacket";

export default (service: ServiceMeta, packet: DiscordPacket<any>) => {
    if (!packet.d) {
        return;
    }
    console.log("MESSAGE Create packet received");
};

import { type ServiceMeta } from "~/service";
import { type DiscordPacket } from "../packets/BasePacket";

export default (service: ServiceMeta, packet: DiscordPacket<any>) => {
    if (!packet.d) {
        return;
    }
    console.log("MESSAGE Create packet received");
};

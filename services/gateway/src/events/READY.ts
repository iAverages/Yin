import { ServiceMeta } from "~/service";
import { DiscordPacket } from "../packets/BasePacket";

export default (service: ServiceMeta, _packet: DiscordPacket<any>) => {
    // core.redis.publish(DiscordEvents.READY, packet);
    console.log("Ready packet recevied");
};

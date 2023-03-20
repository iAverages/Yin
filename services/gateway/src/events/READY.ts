import { DiscordPacket } from "../packets/BasePacket";

export default (_packet: DiscordPacket<any>) => {
    // core.redis.publish(DiscordEvents.READY, packet);
    console.log("Ready packet recevied");
};

export const ready = "ready";

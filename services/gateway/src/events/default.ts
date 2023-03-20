import { DiscordPacket } from "../packets/BasePacket";

export default (_packet: DiscordPacket<any>) => {
    console.log("No handler found for this event.");
};

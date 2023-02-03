import { DiscordPacket } from "../packets/BasePacket";

export default (packet: DiscordPacket<any>) => {
    if (!packet.d) {
        return;
    }
    console.log("Guild Create packet received");
    // const guild = new Guild(packet.d);
    // core.redis.publish("dwad", packet.d);
    // core.emit("guildCreate", guild);
};

import { Client } from "../Client";
import { DiscordPacket } from "../packets/BasePacket";

export const events: Record<string, Promise<{ default: (core: Client, packet: DiscordPacket<any>) => void }>> = {
    MESSAGE_CREATE: import("./MESSAGE_CREATE"),
};

export default events;

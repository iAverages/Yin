import { ServiceMeta } from "~/service";
import { DiscordPacket } from "../packets/BasePacket";

export const events: Record<
    string,
    Promise<{ default: (service: ServiceMeta, packet: DiscordPacket<any>) => void }>
> = {
    MESSAGE_CREATE: import("./MESSAGE_CREATE"),
    READY: import("./READY"),
    GUILD_CREATE: import("./GUILD_CREATE"),
    INTERACTION_CREATE: import("./INTERACTION_CREATE"),
};

export default events;

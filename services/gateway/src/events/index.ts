import { type DiscordPacket } from "~/packets/BasePacket";
import { type ServiceMeta } from "~/service";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type Event<T extends Record<string, any> = object> = {
    service: ServiceMeta;
    packet: DiscordPacket<T>;
};

export const events: Record<
    string,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    Promise<{ default: (event: Event<any>) => Promise<unknown> | unknown }>
> = {
    MESSAGE_CREATE: import("./MESSAGE_CREATE"),
    READY: import("./READY"),
    GUILD_CREATE: import("./GUILD_CREATE"),
    GUILD_UPDATE: import("./GUILD_UPDATE"),
    INTERACTION_CREATE: import("./INTERACTION_CREATE"),
};

export default events;

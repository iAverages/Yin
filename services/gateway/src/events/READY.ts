import { type ServiceMeta } from "~/service";
import { type DiscordPacket } from "../packets/BasePacket";

export default (service: ServiceMeta, _packet: DiscordPacket<any>) => {
    // service.services.database.logEvent(
    //     {
    //         createdAt: new Date(),
    //         discordEvent: "READY",
    //         discordGuildId: "",
    //         discordUserId: "",
    //     },
    //     () => {}
    // );
};

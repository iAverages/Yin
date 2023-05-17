import api from "@yin/discord";

import { ServiceMeta } from "~/service";
import { DiscordPacket } from "../packets/BasePacket";

// import { InteractionResponseType } from "@yin/discord";

export default async (service: ServiceMeta, packet: DiscordPacket<any>) => {
    if (!packet.d) {
        return;
    }
    console.log(packet);
    const res = await api.interaction.respond(
        {
            type: 4,
            data: {
                content: "Hello world!",
            },
        },
        {
            "interaction.id": packet.d.id,
            "interaction.token": packet.d.token,
        }
    );
    console.log(res);
};

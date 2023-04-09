import { DiscordPacket } from "../packets/BasePacket";
import api from "@yin/discord";

// import { InteractionResponseType } from "@yin/discord";

export default async (packet: DiscordPacket<any>) => {
    if (!packet.d) {
        return;
    }
    // api.user.getCurrentUser();
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

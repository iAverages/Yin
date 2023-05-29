import { type GuildSchema } from "@yin/discord/src/structs/guild";

export interface ReadyPacket {
    v: string;
    user: Object; //DiscordUser
    guilds: Array<GuildSchema>; // Array<Guilds>
    session_id: string;
    shard?: [number, number];
    application: Object; // Application
}

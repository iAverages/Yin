export interface ReadyPacket {
    v: string;
    user: Object; //DiscordUser
    guilds: Array<Object>; // Array<Guilds>
    session_id: string;
    shard?: [number, number];
    application: Object; // Application
}

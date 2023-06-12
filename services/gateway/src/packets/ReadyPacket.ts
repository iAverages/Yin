import { type user } from "@yin/discord";

export interface ReadyPacket {
    v: string;
    user: user.User;
    guilds: Array<{ unavailable: boolean; id: string }>;
    session_id: string;
    shard?: [number, number];
    application: { id: string; flags: number };
    resume_gateway_url: string;
}

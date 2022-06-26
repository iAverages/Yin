export interface DiscordPacket<T extends Object = {}> {
    d?: T;
    op: number;
    s?: number;
    t?: string;
}

export interface DiscordPacket<T extends Object = { _yinReqId?: string; _yinProcessStart?: string }> {
    d?: T;
    op: number;
    s?: number;
    t?: string;
}

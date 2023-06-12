// eslint-disable-next-line @typescript-eslint/no-explicit-any
export interface DiscordPacket<T extends Record<string, any> = { _yinReqId?: string; _yinProcessStart?: string }> {
    d?: T;
    op: number;
    s?: number;
    t?: string;
}

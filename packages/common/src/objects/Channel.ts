export interface ChannelPacket {}
export class Channel {
    constructor(packet: ChannelPacket) {
        console.log(packet);
    }

    async sendMessage(): Promise<void> {}
}

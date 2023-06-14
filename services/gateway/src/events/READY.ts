import { logger } from "@yin/common";

import { type Event } from "~/events";
import { type ReadyPacket } from "~/packets/ReadyPacket";

export default ({ packet }: Event<ReadyPacket>) => {
    if (packet.d?.resume_gateway_url) {
        logger.info("Setting new resume url", packet.d.resume_gateway_url);
        return packet.d.resume_gateway_url;
    }
    return null;
};

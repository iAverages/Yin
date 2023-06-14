import { logger } from "@yin/common";
import { message } from "@yin/discord";

import { type Event } from "~/events";

export default async ({ service, packet }: Event<message.Message>) => {
    if (!packet.d) {
        return;
    }

    const messageEvent = await message.messageSchema.parseAsync(packet.d);

    // service.services.worker.handleMessageCreate(
    //     {
    //         author: {
    //             avatar: messageEvent.author.avatar ?? "",
    //             id: messageEvent.author.id,
    //             name: messageEvent.author.username,
    //         },
    //         channelId: messageEvent.channel_id,
    //         content: messageEvent.content,
    //         id: messageEvent.id,
    //         timestamp: messageEvent.timestamp,
    //         webhookId: messageEvent.webhook_id,
    //     },
    //     (err, res) => {
    //         if (err) {
    //             logger.error(err);
    //             throw err;
    //         }
    //         logger.debug(res ? "Added message" : "Failed to add message");
    //     }
    // );
};

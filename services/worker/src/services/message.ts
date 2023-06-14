import api from "@yin/discord";

import { type InternalService } from "~/grpc";

export const createMessageService: InternalService<"handleMessageCreate" | "handleInteraction"> = ({
    logger,
    service,
}) => ({
    handleMessageCreate: async (call, callback) => {
        if (call.request.author) {
            logger.debug("Author found, adding user to database");
            service.services.database.addUser(
                {
                    avatar: call.request.author.avatar,
                    id: call.request.author.id,
                    name: call.request.author.name,
                },
                (err, res) => {
                    if (err) {
                        logger.error(err);
                        callback(err);
                        return;
                    }
                    logger.debug(res ? "Added user" : "Failed to add user");
                    callback(null, { success: true, message: "Added user" });
                }
            );
        }
    },
    handleInteraction: async (call, callback) => {
        const response = await api.interaction.respond(
            {
                type: 4,
                data: {
                    content: `Pong! ${call.request.websocketInfo?.ping}ms`,
                },
            },
            {
                "interaction.id": call.request.id,
                "interaction.token": call.request.token,
            }
        );

        callback(null, {
            success: response.success,
        });
    },
});

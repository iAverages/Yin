import { type InternalService } from "~/grpc";
import { interactions } from "~/interactions";
import { createInteraction } from "~/structs/interaction";

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
        try {
            if (!call.request.data) {
                logger.error("No data found in request");
                callback(null, { success: false, message: "No data found in request" });
                return;
            }

            const interactionHandler = interactions[call.request.data.name];
            const options = JSON.parse(call.request.data.options ?? "[]");
            const hasSubcommand = options.length ?? 0 > 0;

            if (interactionHandler) {
                if (hasSubcommand) {
                    const subcommandName = options[0].name;
                    const subcommandHandler = interactionHandler[subcommandName];
                    if (subcommandHandler) {
                        const interaction = createInteraction(call.request, subcommandHandler.meta);
                        subcommandHandler(interaction, callback);
                        return;
                    }
                }
                const interaction = createInteraction(call.request, interactionHandler.default.meta);
                interactionHandler.default(interaction, callback);
                return;
            }
        } catch (error) {
            logger.error(error, "errored while handinlg interaction?????");
        }
    },
});

import { type InternalService } from "~/grpc";

export const createMessageService: InternalService<"handleMessageCreate"> = ({ logger, service }) => ({
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
                    callback({ message: "Error Test", name: "TEST" }, { success: true, message: "Added user" });
                }
            );
        }
    },
});

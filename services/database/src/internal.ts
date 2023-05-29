import { type InternalService } from "~/grpc";

export const createInternalService: InternalService<"logEvent"> = ({ prisma, logger }) => ({
    logEvent: async ({ request: event }, callback) => {
        try {
            const data = await prisma.eventLog.create({
                data: {
                    createdAt: new Date(),
                    discordEvent: {
                        connect: {
                            name: event.discordEvent,
                        },
                    },
                    guild: {
                        connect: {
                            id: event.discordGuildId,
                        },
                    },
                    user: {
                        connect: {
                            id: event.discordUserId,
                        },
                    },
                },
            });

            console.log(data);
            callback(null, { success: true });
        } catch (err) {
            logger.error(err);
            callback({ message: "Failed to update event log", name: "RECORD_CREATE_FAILURE" }, { success: false });
        }
    },
});

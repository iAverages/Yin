import { type InternalService } from "~/grpc";

export const createUserService: InternalService<"addUser" | "removeUser" | "getUser"> = ({ prisma, logger }) => ({
    addUser: async (call, callback) => {
        try {
            await prisma.discordUser.upsert({
                create: {
                    name: call.request.name,
                    avatar: call.request.avatar,
                },
                update: {
                    name: call.request.name,
                    avatar: call.request.avatar,
                },
                where: {
                    id: call.request.id,
                },
            });
            logger.info(`Added user ${call.request.id} ${call.request.name}`);
            callback(null, { success: true });
        } catch (err) {
            logger.error(err);
            callback({ message: "Failed to add guild", name: "RECORD_CREATE_FAILURE" }, { success: false });
        }
    },

    getUser: async (call, callback) => {
        try {
            const data = await prisma.discordUser.findUnique({
                where: {
                    id: call.request.id,
                },
            });
            if (!data) {
                callback({ message: "Failed to find user", name: "RECORD_NOT_FOUND" }, null);
                return;
            }
            callback(null, { ...data, avatar: data.avatar ?? "" });
        } catch (err) {
            logger.error(err);
            callback({ message: "Failed to get guild", name: "RECORD_READ_FAILURE" }, null);
        }
    },

    removeUser: async (call, callback) => {
        try {
            const data = await prisma.discordUser.delete({
                where: {
                    id: call.request.id,
                },
            });

            callback(null, {
                success: !!data,
                message: data
                    ? `Deleted discord user ${call.request.id}`
                    : `Failed to find discord user ${call.request.id}`,
            });
        } catch (err) {
            logger.error(err);
            callback({ message: "Failed to remove discord user", name: "RECORD_DELETION_FAILURE" }, { success: false });
        }
    },
});

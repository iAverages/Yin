import { InternalService } from "~/grpc";

export const createGuildService: InternalService<"addGuild" | "removeGuild" | "getGuild"> = ({ prisma, logger }) => ({
    addGuild: async (call, callback) => {
        try {
            const data = await prisma.guild.upsert({
                create: {
                    icon: call.request.icon,
                    id: call.request.id,
                    name: call.request.name,
                },
                update: {
                    icon: call.request.icon,
                    id: call.request.id,
                    name: call.request.name,
                },
                where: {
                    id: call.request.id,
                },
            });

            console.log(data);
            callback(null, { success: true });
        } catch (err) {
            logger.error(err);
            callback({ message: "Failed to add guild", name: "RECORD_CREATE_FAILURE" }, { success: false });
        }
    },

    getGuild: async (call, callback) => {
        try {
            const data = await prisma.guild.findUnique({
                where: {
                    id: call.request.id,
                },
            });
            if (!data) {
                callback({ message: "Failed to find guild", name: "RECORD_NOT_FOUND" }, null);
                return;
            }
            callback(null, {
                icon: data.icon ?? "",
                id: data.id,
                name: data.name,
            });
        } catch (err) {
            logger.error(err);
            callback({ message: "Failed to get guild", name: "RECORD_READ_FAILURE" }, null);
        }
    },

    removeGuild: async (call, callback) => {
        try {
            const data = await prisma.guild.delete({
                where: {
                    id: call.request.id,
                },
            });

            callback(null, {
                success: !!data,
                message: !!data ? `Deleted guild ${call.request.id}` : `Failed to find guild ${call.request.id}`,
            });
        } catch (err) {
            logger.error(err);
            callback({ message: "Failed to remove guild", name: "RECORD_DELETION_FAILURE" }, { success: false });
        }
    },
});

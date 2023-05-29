import { logger } from "@yin/common";
import { type DiscordEvent, prisma } from "@yin/db";

const discordEvents: Omit<DiscordEvent, "id">[] = [
    {
        name: "INTERACTION_CREATE",
        description: "An interaction was created, such as a slash command.",
    },
];

export const prepareDatabase = async () => {
    logger.info("Connecting to database");
    await prisma.$connect();
    logger.info("Connected!");

    logger.info("Preparing database state");

    await prisma.$transaction(
        discordEvents.map((event) =>
            prisma.discordEvent.upsert({
                where: { name: event.name },
                update: {},
                create: event,
            })
        )
    );

    logger.info("Complete database preparation");
};

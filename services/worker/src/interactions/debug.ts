import { env } from "~/env";
import type { InteractionHandler } from "~/interactions";

export const info: InteractionHandler = async (interaction, callback) => {
    const randomDelay = Math.floor(Math.random() * 2500);
    await new Promise((resolve) => setTimeout(resolve, randomDelay));
    const response = await interaction.reply({
        embeds: [
            {
                type: "rich",
                title: `Yin Debug`,
                description: "",
                color: 0xb700ff,
                fields: [
                    {
                        name: `Gateway`,
                        value: `\`${interaction.gatewayMeta.pod}\``,
                        inline: true,
                    },
                    {
                        name: `Worker`,
                        value: `\`${env.K3S_POD_NAME}\``,
                        inline: true,
                    },
                    {
                        name: `Random Delay`,
                        value: `\`${randomDelay}MS\``,
                        inline: true,
                    },
                ],
            },
        ],
    });

    callback(null, {
        success: response.success,
    });
};

info.meta = {
    name: "info",
    description: "Get debug info",
};

export const echo: InteractionHandler = async (interaction, callback) => {
    console.log("echoed?");
    console.log(JSON.stringify(interaction.data.options, null, 4));

    const response = await interaction.reply({
        content: interaction.data.options.options?.[0].value ?? "No value",
    });

    callback(null, {
        success: response.success,
    });
};

echo.meta = {
    name: "echo",
    description: "Echo back",
    options: [
        {
            name: "text",
            description: "Text to echo",
            type: "STRING",
            required: true,
        },
    ],
};

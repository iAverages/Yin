import { env } from "~/env";
import type { InteractionHandler } from "~/interactions";

export const info: InteractionHandler = async (interaction, callback) => {
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
                        value: `${interaction.gatewayMeta.pod}`,
                        inline: true,
                    },
                    {
                        name: `Worker`,
                        value: `${env.K3S_POD_NAME}`,
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

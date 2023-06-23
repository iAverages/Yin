import type { InteractionHandler } from "~/interactions";

const def: InteractionHandler = async (interaction, callback) => {
    const response = await interaction.reply({
        embeds: [
            {
                type: "rich",
                title: "Pong!",
                description: interaction.gatewayMeta?.websocketMeta?.ping
                    ? `💓 Heartbeat Latency: **${interaction.gatewayMeta?.websocketMeta?.ping}MS**`
                    : "An error occrued while getting ping.",
                color: 0xb700ff,
            },
        ],
    });

    callback(null, {
        success: response.success,
    });
};

def.meta = {
    name: "ping",
    description: "Display the bots ping to Discord",
};

export default def;

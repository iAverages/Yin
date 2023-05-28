// import * as guilds from "./guilds";
import { interaction, user } from "./structs";

export * from "./structs";

const api = {
    user: user.userHandler,
    interaction: interaction.interactionHandler,
};

export default api;

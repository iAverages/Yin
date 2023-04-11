// import * as guilds from "./guilds";
import { interaction, user } from "./structs";
export { InteractionResponseType } from "./structs/interaction";

export default {
    user: user.handler,
    interaction: interaction.handler,
};

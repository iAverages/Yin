import Discord from "discord.js";
import { version } from "../package.json";

require("dotenv").config({ path: `${__dirname}/.env` });

const client = new Discord.Client({ partials: ["MESSAGE", "CHANNEL", "REACTION"] });

client.login(process.env.BOT_TOKEN);

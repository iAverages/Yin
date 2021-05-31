import path from "path";
import { Yin } from "./Yin";
require("dotenv").config({ path: path.join(`${__dirname}/../.env`) });
const bot = new Yin();
bot.start(process.env.BOT_TOKEN);

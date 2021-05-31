import { Message } from "discord.js";
import { Yin } from "../../Yin";

export const run = (yin: Yin, message: Message, args: string[]) => {
    console.log("Working");
    message.reply("Hello");
};

export const name = "dan";

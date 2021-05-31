import { Message } from "discord.js";
import { Yin } from "../Yin";

export const run = async (yin: Yin, message: Message) => {
    const { author, content } = message;
    if (author.bot) return;

    const a = yin.commands.get(content.split(" ")[0]);
    console.log(a);
    if (a) {
        a.run(yin, message, content.split(" "));
    }
};

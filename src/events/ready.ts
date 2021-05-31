import { Message } from "discord.js";
import { Yin } from "../Yin";

export const run = async (yin: Yin, message: Message) => {
    yin.log.success(`${yin.shard.ids[0]} is ready.`);
};

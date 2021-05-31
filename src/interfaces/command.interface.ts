import { Message } from "discord.js";
import { Yin } from "../Yin";

export interface Run {
    (client: Yin, message: Message, args: string[], alias?: string): Promise<void>;
}

export interface Command {
    name: string;
    aliases: string[];
    run: Run;
}

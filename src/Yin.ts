import { Client, Collection, Intents } from "discord.js";
import { Event } from "./interfaces/event.interface";
import { Command } from "./interfaces/command.interface";
import logger from "@iaverage/logger";
import glob from "glob";
import { promisify } from "util";
import { basename, extname } from "path";
import { Config } from "./config/Config";
import { IConfig } from "config/IConfig";

const globp = promisify(glob);

export class Yin extends Client {
    public log = logger;
    public config: IConfig;
    public commands: Collection<String, Command> = new Collection();
    public aliases: Collection<String, Command> = new Collection();
    public events: Collection<String, Event> = new Collection();

    public constructor() {
        super({ intents: [Intents.FLAGS.GUILD_MESSAGES], partials: ["MESSAGE", "CHANNEL", "REACTION"] });
    }

    public async start(token: string) {
        this.config = new Config(this).config;
        this.loadCommands();
        this.loadEvents();
        this.login(token);
    }

    private async loadCommands() {
        const files = await globp(__dirname + "/commands/**/*.js");
        files.map(async (filePath: string) => {
            const cmd: Command = await import(filePath);
            this.commands.set(cmd.name, cmd);
            cmd.aliases &&
                cmd.aliases.forEach((alias) => {
                    this.aliases.set(alias, cmd);
                });
            this.log.info(`Loaded ${cmd.name} command`);
        });
    }

    private async loadEvents() {
        const files = await globp(__dirname + "/events/**/*.js");
        files.forEach(async (filePath: string) => {
            const event: Event = await import(filePath);
            const eventName = basename(filePath, extname(filePath));
            this.events.set(eventName, event);
            this.on(eventName, event.run.bind(null, this));
            this.log.info(`Loaded ${eventName} event`);
        });
    }
}

import { ShardingManager } from "discord.js";
import log from "./helpers/logger";
import * as _ from "./api/server"; // Start API server
import path from "path";
require("dotenv").config({ path: path.join(`${__dirname}/../.env`) });

// Shard manager
const manager = new ShardingManager(`${__dirname}/bot.js`, { token: process.env.BOT_TOKEN });
manager.on("shardCreate", (shard) => log.info(`Launched new shard ${shard.id}`));
manager.spawn();

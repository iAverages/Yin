import { ShardingManager } from "discord.js";

require("./api/server"); // Start API server

// Shard manager
const manager = new ShardingManager("./bot.js", { token: process.env.BOT_TOKEN });

manager.on("shardCreate", (shard) => console.log(`Launched new shard ${shard.id}`));
manager.spawn();

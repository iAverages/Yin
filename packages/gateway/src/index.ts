import "source-map-support/register";
// import { Guild, Message } from "@yin/common";
import { Client } from "./Client";

const client = new Client();
client.getWebsocket().connect();

// client.on("messageCreate", (message: Message) => {
//     console.log(message);
//     console.log("Got message create event");
// });

// client.on("ready", () => {
//     console.log("READY!!");
// });

// client.on("guildCreate", (guild: Guild) => {
//     console.log("Created Guild");
//     console.log(guild);
// });

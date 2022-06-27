import { Message } from "@yin/common";
import { Client } from "./Client";

const client = new Client();
client.getWebsocket().connect();

client.on("messageCreate", (message: Message) => {
    console.log(message);
    console.log("Got message create event");
});

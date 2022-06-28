import "source-map-support/register";

import { createClient } from "redis";
import { Message } from "@yin/common";

const client = createClient({ url: "redis://redis:6379" });

client.on("error", (err) => console.log("Failed to connect to Redis server", err));

await client.connect();

const subscriber = client.duplicate();

await subscriber.connect();
await subscriber.subscribe("1.0.0:messageCreate", (data: string) => {
    console.log(`[1.0.0:messageCreate] ${data}`);
    if (data === "Testing listeniner") return;
    const message = new Message(JSON.parse(data));
    console.log(message);
});

await client.publish("1.0.0:messageCreate", "Testing listeniner");

import "source-map-support/register";

import { Service, ServiceType } from "@yin/common";

export class Worker extends Service {
    constructor() {
        super(ServiceType.WORKER, "1.0.0");
        this.registerService();
        this.redis.subscribe("1.0.0:messageCreate", (data) => {
            console.log(data);
        });
    }
}

console.log("");
// import { Message, Redis } from "@yin/common";
// import log from "@iaverage/logger";

// await client.connect();

// // client.on("error", (error) => {
// //     log.error("Failed to connect to Redis");
// //     log.error(error);
// //     process.exit(1);
// // });

// client.subscribe("1.0.0:messageCreate", (data: string) => {
//     console.log(`[1.0.0:messageCreate] ${data}`);
//     if (data === "Testing listeniner") return;
//     const message = new Message(JSON.parse(data));
//     console.log(message);
// });

// await client.publish("1.0.0:messageCreate", "Testing listeniner");

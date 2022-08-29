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

new Worker();
console.log("");
// import { Message, Redis } from "@yin/common";
// import log from "@iaverage/logger";

// await client.connect();

// // client.on("error", (error) => {
// //     log.error("Failed to connect to Redis");
// //     log.error(error);
// //     process.exit(1);
// // });

// echo "Building image..."
// docker build -t localhost:5000/yinbot:latest .
// echo "Pushing image to localhost:5000"
// docker image push localhost:5000/yinbot

// client.subscribe("1.0.0:messageCreate", (data: string) => {
//     console.log(`[1.0.0:messageCreate] ${data}`);
//     if (data === "Testing listeniner") return;
//     const message = new Message(JSON.parse(data));
//     console.log(message);
// });

// await client.publish("1.0.0:messageCreate", "Testing listeniner");

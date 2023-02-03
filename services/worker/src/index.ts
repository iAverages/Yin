import "source-map-support/register";
import axios from "axios";
import { DiscordEvents, Service, ServiceType } from "@yin/common";

export class Worker extends Service {
    constructor() {
        super(ServiceType.WORKER, "1.0.0");
        this.registerService();
        const messageHandle = (id: string, _data: any) => {
            // const data = JSON.parse(_data);
            // const processNs = process.hrtime.bigint() - BigInt(data.time);
            // console.log("aaaaaaaaaaaaaaaaaaa", _data);
            // console.log(processNs.toString());
            const data = JSON.parse(_data.message.data);
            if (!data.author) return;
            if (data.author.bot) {
                console.log(`Bot messages, responsed to gateway, not haandling request`);
                // 50 ms delay, sometimes gateway is not listening on channel in time
                // 50 ms is enough delay to ensure gateway is listening
                this.redis.publish(data._yinResponseId, "0", 50);
                return;
            }
            console.log(data);
            if (data.content.startsWith("-d")) {
                this.handleDebug(data);
            }

            if (data.content.startsWith("-s")) {
                this.handleServices(id, data);
            }
        };

        this.redis.stream(DiscordEvents.MESSAGE_CREATE, messageHandle);
    }

    public async handleServices(id: string, data: Record<string, any>) {
        try {
            const a = await this.redis.getRegisteredServices();
            const formatted = [];
            for (const service of a) {
                console.log(service);
                formatted.push({
                    name: `${service.service} - ${service.version}`,
                    value: `${service.uuid}`,
                    inline: true,
                });
            }
            const processNs = process.hrtime.bigint() - BigInt(data._yinProcessStart);
            axios.post(
                `https://discord.com/api/v8/channels/${data.channel_id}/messages`,
                {
                    channel_id: `${data.channel_id}`,
                    content: "",
                    tts: false,
                    embeds: [
                        {
                            type: "rich",
                            title: `Service Info`,
                            description: "",
                            color: 0x633194,
                            fields: formatted,
                            footer: {
                                text: `Processed in ${processNs} NS`,
                            },
                        },
                    ],
                },
                {
                    headers: {
                        Authorization: `Bot ${process.env.YIN_TOKEN}`,
                    },
                }
            );
        } catch (e) {}
    }

    public handleDebug(data: Record<string, any>) {
        const processNs = process.hrtime.bigint() - BigInt(data._yinProcessStart);
        axios.post(
            `https://discord.com/api/v8/channels/${data.channel_id}/messages`,
            {
                channel_id: `${data.channel_id}`,
                content: "",
                tts: false,
                embeds: [
                    {
                        type: "rich",
                        title: `Debug Info`,
                        description: "",
                        color: 0x633194,
                        fields: [
                            {
                                name: `Worker ID`,
                                value: `${this.uuid}`,
                                inline: true,
                            },
                            {
                                name: `Gateway ID`,
                                value: `${data._yinGatewayId ?? "Unknown"}`,
                                inline: true,
                            },
                            {
                                name: `Request ID`,
                                value: `${data._yinReqId ?? "Unknown"}`,
                                inline: true,
                            },
                        ],
                        footer: {
                            text: `Processed in ${processNs} NS`,
                        },
                    },
                ],
            },
            {
                headers: {
                    Authorization: `Bot ${process.env.YIN_TOKEN}`,
                },
            }
        );
        this.redis.publish(data._yinResponseId, "1");
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

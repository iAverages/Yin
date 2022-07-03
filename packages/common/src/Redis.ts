import { createClient } from "redis";
import { Base } from "./Base";

export class Redis extends Base {
    private client;
    #subscribers: Map<string, any>;

    constructor() {
        super();
        this.client = createClient({ url: "redis://redis:6379" });
        this.client.on("error", (err) => this.emit("error", err));
        this.#subscribers = new Map();
    }

    async connect() {
        await this.client.connect();
        this.log.success("Connected to Redis");
    }

    async disconnect() {
        this.log.info(`Unsubscribing from all redis pub/sub channels`);
        for (const [event, subscriber] of this.#subscribers.entries()) {
            await subscriber.unsubscribe();
            this.log.debug(`Subscribed to ${event}`);
        }
        this.#subscribers.clear();
    }

    get instance() {
        return this.client;
    }

    async subscribe(event: string, func: (data: string) => void) {
        const subscriber = this.client.duplicate();
        await subscriber.connect();
        this.#subscribers.set(event, subscriber);
        subscriber.subscribe(event, func);
        this.log.debug(`Subscribed to ${event}`);
    }

    async publish(event: string, data: object | string) {
        const json = JSON.stringify(data);
        this.client.publish(event, json);
        this.log.debug(`Publish ${event} with ${json}`);
    }
}

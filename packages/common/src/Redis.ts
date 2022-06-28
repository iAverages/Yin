import { createClient } from "redis";
import { Base } from "./Base";

export class Redis extends Base {
    private client;
    constructor() {
        super();
        this.client = createClient({ url: "redis://redis:6379" });
        this.client.on("error", (err) => console.error("Failed to connect to Redis server", err));
    }

    async connect() {
        await this.client.connect();
    }

    get instance() {
        return this.client;
    }
}

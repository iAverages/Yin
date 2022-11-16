import IoRedis, { Callback } from "ioredis";
import { Base } from "./Base";
import { Consts } from "./Consts";
import RedisStream, { RedisStreamCallBack } from "./RedisStream";
import { Service } from "./Service";

const EXPIRE_SEC = 30;

export class Redis extends Base {
    private service;
    private client;
    #subscribers: Map<string, IoRedis>;

    constructor(service: Service) {
        super();
        this.service = service;
        this.client = new IoRedis(process.env.REDIS_URL ?? "redis://redis:6379"); //createClient({ url: process.env.REDIS_URL ?? "redis://redis:6379" });
        this.#subscribers = new Map();
    }

    async connect() {
        // await this.client.connect();
        this.log.success("Connected to Redis");
    }

    async disconnect() {
        this.log.info(`Unsubscribing from all redis pub/sub channels`);
        for (const [event, subscriber] of this.#subscribers.entries()) {
            await subscriber.unsubscribe();
            this.log.debug(`Unsubscribed from ${event}`);
        }
        this.#subscribers.clear();
        this.client.disconnect();
    }

    get instance() {
        return this.client;
    }

    async subscribe(event: string, func: Callback<unknown>) {
        const subscriber = this.client.duplicate();
        // await subscriber.connect();
        this.#subscribers.set(event, subscriber);
        subscriber.subscribe(event, func);
        this.log.debug(`Subscribed to ${event}`);
    }

    async unsubscribe(event: string) {
        const channel = `${this.service.version}:${event}`;
        const subscriber = this.#subscribers.get(channel);
        if (!subscriber) {
            this.log.error(`Tried to unsubscribe from channel we are not listening on | Chanel: ${event}`);
            return;
        }
        subscriber.unsubscribe(channel);
        this.#subscribers.delete(event);
    }

    async publish(event: string, data: object | string, delay?: number) {
        const call = async () => {
            try {
                this.client.publish(event, JSON.stringify(data));
                this.log.debug(`Published with pub/sub ${event}`);
            } catch (e) {
                console.log(event, e);
            }
        };

        if (delay) {
            setTimeout(call, delay).unref();
            return;
        }
        await call();
    }

    async identify() {
        const key = this.prefix;
        this.log.debug(`Registering in redis ${key}`);
        await this.client.setex(
            key,
            EXPIRE_SEC,
            JSON.stringify({
                service: this.service.type,
                version: this.service.version,
                uuid: this.service.uuid,
            })
        );
    }

    async removeIdentification() {
        const key = this.prefix;
        this.log.debug(`Unregistering from redis ${key}`);
        await this.client.del(key);
    }

    get prefix() {
        return `${Consts.redis.prefix}:service:${this.service.type.toLowerCase()}:${this.service.uuid}`;
    }

    async stream(event: string, messageHandle: RedisStreamCallBack) {
        const redisStream = new RedisStream(this.service.uuid, this.client);
        redisStream.subscribe(event, messageHandle);
        return redisStream;
    }

    async streamAdd(data: object) {
        try {
            await this.client.xadd(Consts.redis.stream, "*", "data", JSON.stringify(data));
        } catch (e) {
            this.log.error(`Failed to publish data to stream`);
            this.log.error(data);
        }
    }

    public async getRegisteredServices() {
        const res = (await this.client.scan(0, "MATCH", "yin:service:*"))[1];
        const services = new Set<any>();
        for (const id of res) {
            const service = await this.client.get(id);
            if (!service) continue;
            services.add(JSON.parse(service));
        }
        return services;
    }
}

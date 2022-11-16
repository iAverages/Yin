import { Base } from "./Base";
import IoRedis from "ioredis";
import { Consts } from "./Consts";

// TODO: Change the any type here to values JSON can hold?
export type RedisStreamCallBack = (id: string, data: Record<string, any>) => Promise<void> | void;

export default class RedisStream extends Base {
    private id: string;
    private redis: IoRedis;
    private subscribed;

    constructor(id: string, redis: IoRedis) {
        super();
        this.id = id;
        this.redis = redis;
        this.subscribed = false;
    }

    async subscribe(group: string, func: RedisStreamCallBack) {
        try {
            await this.redis.xgroup("CREATE", Consts.redis.stream, group, "$");
            this.log.success(`Created group ${group} in ${Consts.redis.stream} stream`);
        } catch (e) {
            this.log.info(`Group ${group} already exists in ${Consts.redis.stream} stream`);
        }

        // As much as I hate this, I think this is ok to do.
        this.subscribed = true;
        while (true) {
            if (!this.subscribed) {
                this.log.debug(`Unsubscribing from ${group}`);
                break;
            }

            const response = (await this.redis.xreadgroup(
                "GROUP",
                group,
                this.id,
                "COUNT",
                1,
                "BLOCK",
                5000,
                "STREAMS",
                Consts.redis.stream,
                ">"
            )) as Array<Array<[string, [string, string]]>>;

            if (response && response.length > 0) {
                const a = response[0][1];
                this.log.debug(`${this.id}: Processing ${response.length} messages.`);
                for (const message of a) {
                    const id = message[0];
                    const data = message[1][1];
                    console.log(id, data);
                    // fix chaing this from redis to ioredis
                    // format here is different
                    await func(id, JSON.parse(data));
                    const ack = await this.redis.xack(Consts.redis.stream, group, id);
                    this.log.info(`${this.id}: ${ack === 1 ? "Acknowledged" : "Error acknowledging"} processing of ${id}.`);
                }
                console.log("end loop");
            }
        }
    }

    unsubscribe() {}

    publish() {}
}

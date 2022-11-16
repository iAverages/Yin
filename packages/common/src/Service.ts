import { Base } from "./Base";
import { ServiceType } from "./ServiceType";
import { randomUUID } from "crypto";
import { Redis } from "./Redis";

const REFRESH_MS = 25 * 1000;

export class Service extends Base {
    public readonly uuid: string;
    public readonly type: ServiceType;
    public readonly version: string;
    private _redis: Redis;
    private heartbeatTimer: NodeJS.Timer | null;

    constructor(type: ServiceType, version: string) {
        super();
        this.uuid = randomUUID();
        this.type = type;
        this.version = version;
        this._redis = new Redis(this);
        this._redis.connect();
    }

    public async registerService() {
        await this.refreshServiceRegistion();
        this.heartbeatTimer = setInterval(() => this.refreshServiceRegistion(), REFRESH_MS);
    }

    // TODO: Actually use this function on shutdown
    // I need to work out how to get async shut down logic
    // otherwise this will never run correctly
    public async deregister() {
        if (this.heartbeatTimer == null) {
            this.log.warn(`deregister was called but heartbeatTimer timer was null`);
            return;
        }
        clearInterval(this.heartbeatTimer);
        this.heartbeatTimer = null;
        await this._redis.removeIdentification();
    }

    private async refreshServiceRegistion() {
        await this._redis.identify();
    }

    get redis(): Redis {
        return this._redis;
    }
}

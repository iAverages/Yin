import { Base } from "./Base";
import { ServiceType } from "./ServiceType";
import { randomUUID } from "crypto";
import { Redis } from "./Redis";

const REFRESH_MS = 25;

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
        this._redis.idenfity();
        await this.refreshServiceRegistion();
        this.heartbeatTimer = setInterval(this.refreshServiceRegistion, REFRESH_MS);
    }

    public deregister() {
        if (this.heartbeatTimer == null) {
            return;
        }
        clearInterval(this.heartbeatTimer);
        this.heartbeatTimer = null;
    }

    private async refreshServiceRegistion() {
        // this._redis.
    }

    get redis(): Redis {
        return this._redis;
    }
}

import { Base, Redis } from "@yin/common";
import { WebSocket } from "./WebSocket";

export class Client extends Base {
    private websocket: WebSocket;
    private _redis: Redis;

    constructor() {
        super();
        this.websocket = new WebSocket(this);
        this._redis = new Redis();
        this._redis.connect();
    }

    getWebsocket(): WebSocket {
        return this.websocket;
    }

    get redis(): Redis {
        return this._redis;
    }
}

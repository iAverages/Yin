import { Base } from "@yin/common";
import { WebSocket } from "./WebSocket";

export class Client extends Base {
    private websocket: WebSocket;

    constructor() {
        super();
        this.websocket = new WebSocket(this);
    }

    getWebsocket(): WebSocket {
        return this.websocket;
    }
}

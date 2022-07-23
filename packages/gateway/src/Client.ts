import { Service } from "@yin/common";
import { ServiceType } from "@yin/common/src/ServiceType";
import { WebSocket } from "./WebSocket";

export class Client extends Service {
    private websocket: WebSocket;

    constructor() {
        super(ServiceType.GATEWAY, "1.0.0");
        this.websocket = new WebSocket(this);
        this.registerService();
    }

    getWebsocket(): WebSocket {
        return this.websocket;
    }
}

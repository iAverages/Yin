import { Service, ServiceType } from "@yin/common";
import { WebSocket } from "./WebSocket";
import { WorkRequest } from "./WorkRequest";

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

    work(channel: string, data: object) {
        const workRequest = new WorkRequest(this, channel, data);
        workRequest.request();
    }
}

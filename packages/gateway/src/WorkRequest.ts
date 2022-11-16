import { Client } from "./Client";
import { randomUUID } from "crypto";

export class WorkRequest {
    private core: Client;
    private channel: string;
    private data: object;
    private _responded: boolean;
    private responseId: string;

    constructor(core: Client, channel: string, data: object) {
        this.core = core;
        this.channel = channel;
        this.data = data;
        this._responded = false;
        this.responseId = randomUUID();
    }

    public request() {
        this.core.redis.subscribe(this.responseId, (data) => {
            this._responded = true;
        });

        // this.core.redis.publish(this.channel, {
        //     ...this.data,
        //     _yinGatewayId: this.core.uuid,
        //     _yinResponseId: this.responseId,
        // });

        this.core.redis.streamAdd({
            ...this.data,
            _yinGatewayId: this.core.uuid,
            _yinResponseId: this.responseId,
        });
        // If worker does not respond, select another and retry
        setTimeout(() => {
            this.core.redis.unsubscribe(this.responseId);
            this.core.log.debug(`Worker did${this._responded ? "" : " not"} responsed`);
            // TODO: Send message to user saying error occured.
        }, 5000);
    }

    public getResponseId() {
        return this.responseId;
    }
}

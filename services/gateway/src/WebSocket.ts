import { randomUUID } from "crypto";
import WS from "ws";

import { consts, logger } from "@yin/common";

import EventHandlers from "./events";
import { type DiscordPacket } from "./packets/BasePacket";
import { type HelloPacket } from "./packets/HelloPacket";
import { type ReadyPacket } from "./packets/ReadyPacket";
import { type ServiceMeta } from "./service";
import { DiscordEvents, Opcodes } from "./WSConsts";

export class WebSocket {
    private url = consts.discord.gateway;
    private readonly token = process.env.DISCORD_TOKEN;

    private service: ServiceMeta;
    private connection: WS;
    public sessionId: string | null;
    public expectedGuilds: any;
    private sequence = -1;
    private closeSequence = 0;
    public ping = -1;
    public lastPingTime = -1;
    public lastHeartbeat = false;
    private heartbeatTimer: NodeJS.Timer | null;
    public connectedAt = -1;
    public lastHeartbeatAcked = false;

    constructor(service: ServiceMeta) {
        this.service = service;
    }

    onOpen() {
        logger.info("Connected to websocket!");
    }

    onError(error: WS.ErrorEvent) {
        console.log(error);
    }

    onMessage({ data }: WS.MessageEvent) {
        const packet: DiscordPacket = JSON.parse(data as string);
        // console.log(packet);
        if (packet.s && packet.s > this.sequence) {
            this.sequence = packet.s;
        }

        if (packet.t != null) {
            switch (packet.t) {
                case DiscordEvents.READY:
                    const readyData = packet.d as ReadyPacket;
                    this.sessionId = readyData.session_id;
                    this.expectedGuilds = new Set(readyData.guilds.map((d: any) => d.id));
                    logger.info(`Gateway ready, Session ${this.sessionId}.`);
                    this.lastHeartbeatAcked = true;
                    this.sendHeartbeat();
                    this.handlePacket(packet);
                    break;

                case DiscordEvents.RESUMED:
                    break;
            }
        }

        switch (packet.op) {
            case Opcodes.HELLO:
                // this.setHelloTimeout(-1);
                const helloData = packet.d as HelloPacket;
                logger.info("Hello from WS");
                this.setHeartbeatTimer(helloData.heartbeat_interval);
                this.identify();
                break;
            case Opcodes.HEARTBEAT_ACK:
                this.ackHeartbeat();
                break;
            case Opcodes.HEARTBEAT:
                this.sendHeartbeat();
                break;
            case Opcodes.RECONNECT:
                logger.info("WebSocket requested reconnect");
                this.reconnect();
                break;
            case Opcodes.INVALID_SESSION:
                logger.warn(`Invalid session, ${packet.d ? "resuming." : "disconnecting."}.`);
                // Reconnect if true.
                if (packet.d) {
                    this.identifyResume();
                    return;
                }
                this.sequence = -1;
                this.sessionId = null;
                this.disconnect();
                break;
            default:
                // Handle events
                this.handlePacket(packet);
        }
    }

    async handlePacket(packet: DiscordPacket) {
        if (!packet.t) {
            logger.debug(`Recevied packet with no event name`);
            return;
        }

        // @ts-ignore
        if (packet.d?.resumeGatewayUrl) {
            // @ts-ignore
            this.url = packet.d.resumeGatewayUrl;
            // @ts-ignore
            console.log("Setting new resume url", packet.d.resumeGatewayUrl);
        }
        try {
            logger.debug(`Called packet handler for event ${packet.t}`);
            if (!packet.d) {
                packet.d = {};
            }
            packet.d._yinReqId = randomUUID();
            // Maybe capture this info right as we receieve the event from discord
            packet.d._yinProcessStart = process.hrtime.bigint().toString();

            const handler = await EventHandlers[packet.t];
            handler.default(this.service, packet);
        } catch (e) {
            console.log(e);
            logger.error(`Failed to handle ${packet.t} packet.`);
            return;
        }
        logger.info(`Got ${packet.t} packet`);
    }

    identifyResume() {
        if (!this.sessionId) {
            logger.debug("No session id was found, starting new session.");
            this.identify();
            return;
        }

        logger.debug(`Resuming session ${this.sessionId}, sequence ${this.closeSequence}`);

        const d = {
            session_id: this.sessionId,
            seq: this.closeSequence,
            token: this.token,
        };

        this.send({ op: Opcodes.RESUME, d });
    }

    onClose(close: WS.CloseEvent) {
        close.wasClean && logger.warn("Disconnected from websocket! Reason: " + close.reason);
        !close.wasClean && logger.warn("Bad disconnect from websocket! Reason: " + close.reason);
    }

    ackHeartbeat() {
        this.lastHeartbeatAcked = true;
        const ping = Date.now() - this.lastPingTime;
        logger.debug(`Heartbeat acknowledged - ${ping}ms ping.`);
        this.ping = ping;
    }

    connect() {
        if (this.connection && this.connection.readyState == 1) {
            return;
        }
        const ws = new WS(this.url);
        this.connection = ws;
        ws.onopen = this.onOpen.bind(this);
        ws.onmessage = this.onMessage.bind(this);
        ws.onerror = this.onError.bind(this);
        ws.onclose = this.onClose.bind(this);
    }

    disconnect() {
        this.connection.close();
    }

    identify() {
        const d = {
            intents: 3276799, // TODO: Change this later on, this number is everything for now
            token: this.token,
            properties: {
                os: "linux",
                browser: "Yin",
                device: "Yin",
            },
        };
        this.send({ op: Opcodes.IDENTIFY, d });
    }

    reconnect() {
        if (this.connection) {
            this.connection.close();
        }
        this.connect();
    }

    destroy() {}

    send(data: Object) {
        this.connection.send(JSON.stringify(data));
    }

    sendHeartbeat() {
        // Second part of this (After ||) helps with network dropouts. I should
        // Probably think of a better way to handle this but for now this will do :)
        if (!this.lastHeartbeatAcked && this.ping != -1) {
            logger.warn("Last heartbeat did not ack, reconnecting to websocket.");
            this.reconnect();
            return;
        }
        this.lastHeartbeatAcked = false;
        this.lastPingTime = Date.now();
        this.send({ op: Opcodes.HEARTBEAT, d: this.sequence });
    }

    setHeartbeatTimer(time: number) {
        if (time === -1) {
            if (this.heartbeatTimer) {
                logger.debug("Removing the heartbeat.");
                clearInterval(this.heartbeatTimer);
                this.heartbeatTimer = null;
            }
            return;
        }
        if (this.heartbeatTimer) {
            clearInterval(this.heartbeatTimer);
        }
        this.heartbeatTimer = setInterval(() => this.sendHeartbeat(), time).unref();
        logger.info(`Setting heartbeat interval of ${time}ms.`);
    }
}

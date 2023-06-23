import { randomUUID } from "crypto";
import WS from "ws";

import { consts, logger, metrics, SentryNode } from "@yin/common";

import { env } from "~/env";
import EventHandlers from "./events";
import { type DiscordPacket } from "./packets/BasePacket";
import { type HelloPacket } from "./packets/HelloPacket";
import { type ReadyPacket } from "./packets/ReadyPacket";
import { type ServiceMeta } from "./service";
import { DiscordEvents, Opcodes } from "./WSConsts";

export class WebSocket {
    private url = consts.discord.gateway;
    private readonly token = env.YIN_DISCORD_TOKEN;

    private service: ServiceMeta;
    private connection: WS;
    public sessionId: string | null;
    private sequence = -1;
    private closeSequence = 0;
    public ping = -1;
    public lastPingTime = -1;
    public lastHeartbeat = false;
    private heartbeatTimer: NodeJS.Timer | null;
    public connectedAt = -1;
    public lastHeartbeatAcked = false;
    private packetCounter = new metrics.Counter({
        help: "Number of packets received",
        name: "gateway_packets_received",
    });

    constructor(service: ServiceMeta) {
        this.service = service;
    }

    onOpen() {
        logger.info("Connected to websocket!");
    }

    onError(error: WS.ErrorEvent) {
        console.log(error);
    }

    async onMessage({ data }: WS.MessageEvent) {
        this.packetCounter.inc();
        const packet: DiscordPacket = JSON.parse(data as string);
        if (packet.s && packet.s > this.sequence) {
            this.sequence = packet.s;
        }

        if (packet.t != null) {
            switch (packet.t) {
                case DiscordEvents.READY:
                    this.sessionId = (packet.d as ReadyPacket).session_id;
                    logger.info(`Gateway ready, Session ${this.sessionId}.`);
                    this.lastHeartbeatAcked = true;
                    this.sendHeartbeat();
                    this.handlePacket(packet);
                    if (packet.t == DiscordEvents.READY) {
                        const newUrl = (await (
                            await EventHandlers["READY"]
                        ).default({
                            service: this.service,
                            packet,
                            wsInfo: {
                                ping: this.ping,
                            },
                        })) as unknown as string;
                        if (newUrl) {
                            this.url = newUrl;
                            logger.info("Setting new resume url", newUrl);
                        }
                        return;
                    }
                    break;

                case DiscordEvents.RESUMED:
                    break;
            }
        }

        switch (packet.op) {
            case Opcodes.HELLO:
                logger.info("Hello from WS");
                this.setHeartbeatTimer((packet.d as HelloPacket).heartbeat_interval);
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

        try {
            if (!packet.d) {
                packet.d = {};
            }
            packet.d._yinReqId = randomUUID();
            // Maybe capture this info right as we receieve the event from discord
            packet.d._yinProcessStart = process.hrtime.bigint().toString();

            const handler = await EventHandlers[packet.t];
            if (!handler) {
                logger.debug(`No handler for ${packet.t} packet, ignoring...`);
                return;
            }

            logger.debug(`Called packet handler for event ${packet.t}`);
            await handler.default({
                service: this.service,
                packet,
                wsInfo: {
                    ping: this.ping,
                },
            });
        } catch (e) {
            logger.error(`Failed to handle ${packet.t} packet.`);
            logger.error(e);
            SentryNode.captureException(e);
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

    // destroy() {}

    send(data: object) {
        this.connection.send(JSON.stringify(data));
    }

    sendHeartbeat() {
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

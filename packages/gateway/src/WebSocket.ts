import WS from "ws";
import { Opcodes, DiscordEvents } from "./WSConsts";
import { Base } from "./Base";
import { DiscordPacket } from "./packets/BasePacket";
import { HelloPacket } from "./packets/HelloPacket";
import { ReadyPacket } from "./packets/ReadyPacket";

export class WebSocket extends Base {
    // Tmp while testing and experimenting
    private readonly url = "wss://gateway.discord.gg/?v=9&encoding=json";
    private readonly token = "NDc5NTk5MjI5MDYxNDMxMjk4.W3VTlg.OXQDIVaHo9SbosDfv4s44ThNJ1s";

    public id: string;
    private connection: WS;
    public sessionId: string | null;
    public expectedGuilds: any;
    private sequence: number = -1;
    private closeSequence: number = 0;
    public ping: number;
    public lastPingTime: number = -1;
    public lastHeartbeat: boolean = false;
    private heartbeatTimer: NodeJS.Timer | null;
    public connectedAt: number = -1;
    public lastHeartbeatAcked: boolean = false;

    constructor() {
        super();
    }

    onOpen() {
        this.log.success("Connected to websocket!");
    }

    onError(error: WS.ErrorEvent) {
        console.log(error);
    }

    onMessage({ data }: WS.MessageEvent) {
        const packet: DiscordPacket = JSON.parse(data as string);
        if (packet.s && packet.s > this.sequence) {
            this.sequence = packet.s;
        }

        if (packet.t != null) {
            switch (parseInt(packet.t)) {
                case DiscordEvents.READY:
                    const readyData = packet.d as ReadyPacket;
                    this.sessionId = readyData.session_id;
                    this.expectedGuilds = new Set(readyData.guilds.map((d: any) => d.id));
                    this.log.success(`Gateway ready, Session ${this.sessionId}.`);
                    this.lastHeartbeatAcked = true;
                    this.sendHeartbeat();
                    break;

                case DiscordEvents.RESUMED:
                    break;
            }
        }

        switch (packet.op) {
            case Opcodes.HELLO:
                // this.setHelloTimeout(-1);
                const helloData = packet.d as HelloPacket;
                this.log.info("Hello from WS");
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
                this.log.info("WebSocket requested reconnect");
                this.reconnect();
                break;
            case Opcodes.INVALID_SESSION:
                this.log.warn(`Invalid session, ${packet.d ? "resuming." : "disconnecting."}.`);
                // Reconnect if true.
                if (packet.d) {
                    this.identifyResume();
                    return;
                }
                this.sequence = -1;
                this.sessionId = null;
                break;
            default:
                console.log(packet);
            // Handle events
        }
    }

    identifyResume() {
        if (!this.sessionId) {
            this.log.debug("No session id was found, starting new session.");
            this.identify();
            return;
        }

        this.log.debug(`Resuming session ${this.sessionId}, sequence ${this.closeSequence}`);

        const d = {
            session_id: this.sessionId,
            seq: this.closeSequence,
            token: this.token,
        };

        this.send({ op: Opcodes.RESUME, d });
    }

    onClose(close: WS.CloseEvent) {
        this.log.warn("Disconnected from websocket!");
        this.log.warn(close);
    }

    ackHeartbeat() {
        this.lastHeartbeatAcked = true;
        const ping = Date.now() - this.lastPingTime;
        this.log.debug(`Heartbeat acknowledged - ${ping}ms ping.`);
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

    identify() {
        const d = {
            intents: 513,
            token: this.token,
            properties: {
                $os: "linux",
                $browser: "YinBot",
                $device: "YinBot",
            },
        };
        this.send({ op: Opcodes.IDENTIFY, d });
    }

    reconnect() {
        if (this.connection) {
            this.connection.close();
        }
    }

    destroy() {}

    send(data: Object) {
        this.connection.send(JSON.stringify(data));
    }

    sendHeartbeat() {
        this.lastHeartbeatAcked = false;
        this.lastPingTime = Date.now();
        this.send({ op: Opcodes.HEARTBEAT, d: this.sequence });
    }

    setHeartbeatTimer(time: number) {
        if (time === -1) {
            if (this.heartbeatTimer) {
                this.log.debug("Removing the heartbeat.");
                clearInterval(this.heartbeatTimer);
                this.heartbeatTimer = null;
            }
            return;
        }
        if (this.heartbeatTimer) {
            clearInterval(this.heartbeatTimer);
        }
        this.heartbeatTimer = setInterval(() => this.sendHeartbeat(), time).unref();
        this.log.info(`Setting heartbeat interval of ${time}ms.`);
    }
}

new WebSocket().connect();

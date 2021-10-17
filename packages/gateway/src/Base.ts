import log from "@iaverage/logger";
import EventEmitter from "eventemitter3";

export class Base extends EventEmitter {
    private readonly className: string;

    public readonly log = {
        debug: (m: any) => log.uwu(`[${this.className}] ${m}`),
        info: (m: any) => log.info(`[${this.className}] ${m}`),
        error: (m: any) => log.error(`[${this.className}] ${m}`),
        success: (m: any) => log.success(`[${this.className}] ${m}`),
        warn: (m: any) => log.warn(`[${this.className}] ${m}`),
    };

    constructor(className: string) {
        super();
        this.className = className;
    }
}

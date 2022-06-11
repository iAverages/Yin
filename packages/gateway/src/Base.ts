import log from "@iaverage/logger";
import EventEmitter from "eventemitter3";

export class Base extends EventEmitter {
    public readonly log = {
        debug: (m: any) => log.uwu(`[${this.constructor.name}] ${m}`),
        info: (m: any) => log.info(`[${this.constructor.name}] ${m}`),
        error: (m: any) => log.error(`[${this.constructor.name}] ${m}`),
        success: (m: any) => log.success(`[${this.constructor.name}] ${m}`),
        warn: (m: any) => log.warn(`[${this.constructor.name}] ${m}`),
    };
}

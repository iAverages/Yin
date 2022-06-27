export const DISCORD_EPOCH = BigInt(1420070400000);

export default class Snowflake {
    public readonly id: bigint;

    public constructor(id: string | bigint) {
        try {
            this.id = BigInt(id);
        } catch {
            throw TypeError("Invalid Snowflake");
        }
    }

    public getTimestamp(): number {
        return Number((this.id >> 22n) + DISCORD_EPOCH);
    }

    public getWorkerId(): number {
        return Number((this.id >> 17n) & 0b11111n);
    }

    public getProcessId(): number {
        return Number((this.id >> 12n) & 0b11111n);
    }

    public getIncrement(): number {
        return Number(this.id & 0b111111111111n);
    }

    public getDate(): Date {
        return new Date(this.getTimestamp());
    }

    public toJson(): DeconstructedSnowflake {
        return {
            id: this.id,
            timestamp: this.getTimestamp(),
            workerId: this.getWorkerId(),
            processId: this.getProcessId(),
            increment: this.getIncrement(),
            epoch: DISCORD_EPOCH,
        };
    }
}

export interface DeconstructedSnowflake {
    id: bigint;
    timestamp: number;
    workerId: number;
    processId: number;
    increment: number;
    epoch: bigint;
}

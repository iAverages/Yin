import { Yin } from "../Yin";

export interface Run {
    (client: Yin, ...args: any[]): Promise<void>;
}

export interface Event {
    name: string;
    run: Run;
}

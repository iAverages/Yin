import type { worker } from "@yin/grpc";

import type { Interaction } from "~/structs/interaction";
import * as debug from "./debug";
import * as ping from "./ping";

type Handler = (
    interaction: Interaction,
    callback: (error: Error | null, response: worker.StatusReply) => void
) => void;

const ApplicationCommandOptionType = {
    SUB_COMMAND: 1,
    SUB_COMMAND_GROUP: 2,
    STRING: 3,
    INTEGER: 4,
    BOOLEAN: 5,
    USER: 6,
    CHANNEL: 7,
    ROLE: 8,
    MENTIONABLE: 9,
    NUMBER: 10,
    ATTACHMENT: 11,
} as const;

export type InteractionHandler = Handler & {
    meta: {
        name: string;
        description: string;
        options?: [
            {
                name: string;
                description: string;
                type: keyof typeof ApplicationCommandOptionType;
                required?: boolean;
                choices?: [
                    {
                        name: string;
                        value: string;
                    }
                ];
                options?: [];
            }
        ];
    };
};

export const interactions: Record<string, Record<string, InteractionHandler>> = {
    debug: debug,
    ping: ping,
};

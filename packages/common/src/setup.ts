import { uuid } from "uuidv4";

import { type user } from "@yin/discord";

export type Services = "gateway" | "worker";

export type DefaultServiceMeta = {
    uuid: string;
    type: Services;
    botUser: user.User;
};

export const setup = <T>(type: Services, meta: T & Omit<DefaultServiceMeta, "uuid" | "type">) => {
    return {
        uuid: uuid(),
        type,
        ...meta,
    };
};

import { uuid } from "uuidv4";

export type Services = "gateway" | "worker";

export type DefaultServiceMeta = {
    uuid: string;
    type: Services;
};

export const setup = <T>(type: Services, meta: T) => {
    return {
        uuid: uuid(),
        type,
        ...meta,
    };
};

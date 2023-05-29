import { type Routes } from "./routes";
import { type GuildPartialSchema } from "./structs/guild";
import { type InterationResponseUrlParts } from "./structs/interaction";
import { type GetUserUrlParts, type User } from "./structs/user";

// prettier-ignore
export type RequestResponses<T> = 
    T extends Routes.USERS_ME ? User : 
    T extends Routes.USER ? User  : 
    T extends Routes.USERS_ME_GUILDS ? GuildPartialSchema : 
    null;

// prettier-ignore
export type UrlParts<T> = 
    T extends Routes.USER ? GetUserUrlParts :
    T extends Routes.INTERACTION_CREATE ? InterationResponseUrlParts :
    null;

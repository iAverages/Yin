import { Routes } from "./routes";
import { GetUserUrlParts, User } from "./structs/user";
import { GuildPartialSchema } from "structs/guild";
import { InterationResponseUrlParts } from "structs/interaction";

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

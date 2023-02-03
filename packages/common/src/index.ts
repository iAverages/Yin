// Once build logs will show TS file paths
import "source-map-support/register";
import path from "path";

// Always load dotenv when common is imported
import dotenv from "dotenv";
dotenv.config({ path: path.resolve("../../.env") });

import { Message, MessagePacket } from "./objects/Message";
import { Guild, GuildObject } from "./objects/guild/Guild";
import { consts } from "./consts";
import { globalSchema, validateEnvVars } from "./env";

export { Message, MessagePacket, Guild, GuildObject, consts, validateEnvVars, globalSchema };

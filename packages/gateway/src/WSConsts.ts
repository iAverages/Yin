export enum DiscordEvents {
    "READY",
    "RESUMED",
    "APPLICATION_COMMAND_CREATE",
    "APPLICATION_COMMAND_DELETE",
    "APPLICATION_COMMAND_UPDATE",
    "GUILD_CREATE",
    "GUILD_DELETE",
    "GUILD_UPDATE",
    "INVITE_CREATE",
    "INVITE_DELETE",
    "GUILD_MEMBER_ADD",
    "GUILD_MEMBER_REMOVE",
    "GUILD_MEMBER_UPDATE",
    "GUILD_MEMBERS_CHUNK",
    "GUILD_INTEGRATIONS_UPDATE",
    "GUILD_ROLE_CREATE",
    "GUILD_ROLE_DELETE",
    "GUILD_ROLE_UPDATE",
    "GUILD_BAN_ADD",
    "GUILD_BAN_REMOVE",
    "GUILD_EMOJIS_UPDATE",
    "CHANNEL_CREATE",
    "CHANNEL_DELETE",
    "CHANNEL_UPDATE",
    "CHANNEL_PINS_UPDATE",
    "MESSAGE_CREATE",
    "MESSAGE_DELETE",
    "MESSAGE_UPDATE",
    "MESSAGE_DELETE_BULK",
    "MESSAGE_REACTION_ADD",
    "MESSAGE_REACTION_REMOVE",
    "MESSAGE_REACTION_REMOVE_ALL",
    "MESSAGE_REACTION_REMOVE_EMOJI",
    "THREAD_CREATE",
    "THREAD_UPDATE",
    "THREAD_DELETE",
    "THREAD_LIST_SYNC",
    "THREAD_MEMBER_UPDATE",
    "THREAD_MEMBERS_UPDATE",
    "USER_UPDATE",
    "PRESENCE_UPDATE",
    "TYPING_START",
    "VOICE_STATE_UPDATE",
    "VOICE_SERVER_UPDATE",
    "WEBHOOKS_UPDATE",
    "INTERACTION_CREATE",
    "STAGE_INSTANCE_CREATE",
    "STAGE_INSTANCE_UPDATE",
    "STAGE_INSTANCE_DELETE",
    "GUILD_STICKERS_UPDATE",
}

export enum Status {
    READY = 0,
    CONNECTING = 1,
    RECONNECTING = 2,
    IDLE = 3,
    NEARLY = 4,
    DISCONNECTED = 5,
    WAITING_FOR_GUILDS = 6,
    IDENTIFYING = 7,
    RESUMING = 8,
}

export enum Opcodes {
    DISPATCH = 0,
    HEARTBEAT = 1,
    IDENTIFY = 2,
    STATUS_UPDATE = 3,
    VOICE_STATE_UPDATE = 4,
    VOICE_GUILD_PING = 5,
    RESUME = 6,
    RECONNECT = 7,
    REQUEST_GUILD_MEMBERS = 8,
    INVALID_SESSION = 9,
    HELLO = 10,
    HEARTBEAT_ACK = 11,
}

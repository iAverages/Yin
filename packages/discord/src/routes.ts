export enum Routes {
    USERS_ME = "/users/@me",
    USER = "/users/{user.id}",
    USERS_ME_GUILDS = "/users/@me/guilds",

    INTERACTION_CREATE = "/interactions/{interaction.id}/{interaction.token}/callback",
}

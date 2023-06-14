export enum Routes {
    USERS_ME = "/users/@me",
    USER = "/users/{user.id}",
    USERS_ME_GUILDS = "/users/@me/guilds",

    INTERACTION_CREATE = "/interactions/{interaction.id}/{interaction.token}/callback",

    INTERACTION_FOLLOWUP = "/webhooks/{application.id}/{interaction.token}", // same as webhook, but the route parts are different for the TS types
    WEBHOOK = "/webhooks/{webhook.id}/{webhook.token}",
}

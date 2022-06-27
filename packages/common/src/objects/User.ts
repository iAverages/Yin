import Snowflake from "../Snowflake";

export interface UserDiscordObject {
    id: Snowflake;
    username: string;
    discriminator: string;
    avatar: string;
    bot?: boolean;
    system?: boolean;
    mfa_enabled?: boolean;
    banner?: string;
    accent_color?: number;
    locale?: string;
    verified?: boolean;
    email?: string;
    flags?: number;
    premium_type?: number;
    public_flags?: number;
}

export class User {
    public id: Snowflake;
    public username: string;
    public discriminator: string;
    public avatar: string;
    public bot?: boolean;
    public system?: boolean;
    public mfaEnabled?: boolean;
    public banner?: string;
    public accentColor?: number;
    public locale?: string;
    public verified?: boolean;
    public email?: string;
    public flags?: number;
    public premiumType?: number;
    public publicFlags?: number;

    constructor(packet: UserDiscordObject) {
        this.id = packet.id;
        this.username = packet.username;
        this.discriminator = packet.discriminator;
        this.avatar = packet.avatar;
        this.bot = packet.bot;
        this.system = packet.system;
        this.mfaEnabled = packet.mfa_enabled;
        this.banner = packet.banner;
        this.accentColor = packet.accent_color;
        this.locale = packet.locale;
        this.verified = packet.verified;
        this.email = packet.email;
        this.flags = packet.flags;
        // TODO: Make enums for these
        this.premiumType = packet.premium_type;
        this.publicFlags = packet.public_flags;
    }
}

import { User } from "../User";

export interface GuildMemberObject {
    user?: User;
    nick?: string;
    avatar?: string;
    roles: Array<Object>; // TODO: Change to array roles
    joined_at: Date;
    premium_since?: Date;
    deaf: boolean;
    mute: boolean;
    pending?: boolean;
    permissions?: string;
    communication_disabled_until?: Date;
}

export class GuildMember {
    public readonly user?: User;
    public readonly nick?: string;
    public readonly avatar?: string;
    public readonly roles: Array<Object>; // TODO: Change to array
    public readonly joined_at: Date;
    public readonly premium_since?: Date;
    public readonly deaf: boolean;
    public readonly mute: boolean;
    public readonly pending?: boolean;
    public readonly permissions?: string;
    public readonly communication_disabled_until?: Date;

    constructor(packet: GuildMemberObject) {
        this.user = packet.user;
        this.nick = packet.nick;
        this.avatar = packet.avatar;
        this.roles = packet.roles;
        this.joined_at = packet.joined_at;
        this.premium_since = packet.premium_since;
        this.deaf = packet.deaf;
        this.mute = packet.mute;
        this.pending = packet.pending;
        this.permissions = packet.permissions;
        this.communication_disabled_until = packet.communication_disabled_until;
    }
}

-- Bot membership is independent of whether a guild has configured any settings.
CREATE TABLE bot_guilds (
    guild_id BIGINT UNSIGNED NOT NULL PRIMARY KEY,
    name VARCHAR(100) NULL,
    icon VARCHAR(128) NULL,
    shard_id INT UNSIGNED NOT NULL,
    bot_present BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    KEY bot_guilds_shard (shard_id, bot_present)
);

-- OAuth membership/permissions are user-scoped, never inferred from bot membership.
-- Auth warms this snapshot at sign-in; the API refreshes it on expiry.
CREATE TABLE user_guild_cache (
    user_id VARCHAR(36) NOT NULL PRIMARY KEY,
    guilds JSON NOT NULL,
    expires_at TIMESTAMP(3) NOT NULL,
    CONSTRAINT user_guild_cache_user_fk FOREIGN KEY (user_id) REFERENCES `user` (id) ON DELETE CASCADE
);

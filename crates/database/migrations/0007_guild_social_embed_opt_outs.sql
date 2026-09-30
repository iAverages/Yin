CREATE TABLE guild_social_embed_opt_outs (
    guild_id BIGINT UNSIGNED NOT NULL,
    platform VARCHAR(16) NOT NULL,
    PRIMARY KEY (guild_id, platform)
);

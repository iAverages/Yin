CREATE TABLE user_social_embed_opt_outs (
    user_id BIGINT UNSIGNED NOT NULL,
    platform VARCHAR(16) NOT NULL,
    PRIMARY KEY (user_id, platform)
);

CREATE TABLE user_social_embed_displays (
    user_id BIGINT UNSIGNED NOT NULL,
    platform VARCHAR(16) NOT NULL,
    display VARCHAR(16) NOT NULL,
    PRIMARY KEY (user_id, platform)
);

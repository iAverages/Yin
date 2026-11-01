CREATE TABLE user_social_embed_notices (
    user_id BIGINT UNSIGNED NOT NULL,
    notice VARCHAR(16) NOT NULL,
    PRIMARY KEY (user_id, notice)
);

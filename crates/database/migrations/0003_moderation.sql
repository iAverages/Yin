CREATE TABLE moderation_cases (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    guild_id BIGINT UNSIGNED NOT NULL,
    target_user_id BIGINT UNSIGNED NULL,
    target_channel_id BIGINT UNSIGNED NULL,
    actor_user_id BIGINT UNSIGNED NULL,
    source VARCHAR(32) NOT NULL,
    external_audit_log_id BIGINT UNSIGNED NULL,
    action VARCHAR(32) NOT NULL,
    reason TEXT NULL,
    duration_seconds BIGINT UNSIGNED NULL,
    expires_at TIMESTAMP(3) NULL,
    parent_case_id BIGINT UNSIGNED NULL,
    status VARCHAR(16) NOT NULL,
    failure_reason TEXT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    completed_at TIMESTAMP(3) NULL,
    CONSTRAINT moderation_cases_one_target CHECK (
        (target_user_id IS NOT NULL AND target_channel_id IS NULL)
        OR (target_user_id IS NULL AND target_channel_id IS NOT NULL)
    ),
    CONSTRAINT moderation_cases_parent_fk
        FOREIGN KEY (parent_case_id) REFERENCES moderation_cases (id) ON DELETE SET NULL,
    UNIQUE KEY moderation_cases_external_audit (guild_id, external_audit_log_id),
    KEY moderation_cases_user_history (guild_id, target_user_id, created_at, id),
    KEY moderation_cases_channel_history (guild_id, target_channel_id, created_at, id),
    KEY moderation_cases_parent (parent_case_id)
);

CREATE TABLE moderation_case_history (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    case_id BIGINT UNSIGNED NOT NULL,
    status VARCHAR(16) NOT NULL,
    detail TEXT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT moderation_case_history_case_fk
        FOREIGN KEY (case_id) REFERENCES moderation_cases (id) ON DELETE CASCADE,
    KEY moderation_case_history_case (case_id, created_at, id)
);

CREATE TABLE moderation_audit_cursors (
    guild_id BIGINT UNSIGNED NOT NULL PRIMARY KEY,
    highest_audit_entry_id BIGINT UNSIGNED NOT NULL,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)
);

CREATE TABLE moderation_warnings (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    guild_id BIGINT UNSIGNED NOT NULL,
    target_user_id BIGINT UNSIGNED NOT NULL,
    moderator_user_id BIGINT UNSIGNED NOT NULL,
    case_id BIGINT UNSIGNED NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    revoked_at TIMESTAMP(3) NULL,
    revoked_by_user_id BIGINT UNSIGNED NULL,
    revocation_reason TEXT NULL,
    CONSTRAINT moderation_warnings_case_fk
        FOREIGN KEY (case_id) REFERENCES moderation_cases (id) ON DELETE SET NULL,
    KEY moderation_warnings_active (guild_id, target_user_id, revoked_at, created_at),
    KEY moderation_warnings_case (case_id)
);

CREATE TABLE moderation_warning_subjects (
    guild_id BIGINT UNSIGNED NOT NULL,
    target_user_id BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (guild_id, target_user_id)
);

CREATE TABLE punishment_ladder_rules (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    guild_id BIGINT UNSIGNED NOT NULL,
    warning_threshold INT UNSIGNED NOT NULL,
    window_seconds BIGINT UNSIGNED NOT NULL,
    action VARCHAR(32) NOT NULL,
    duration_seconds BIGINT UNSIGNED NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT punishment_ladder_threshold_positive CHECK (warning_threshold > 0),
    CONSTRAINT punishment_ladder_window_positive CHECK (window_seconds > 0),
    UNIQUE KEY punishment_ladder_rule (guild_id, warning_threshold, window_seconds, action)
);

CREATE TABLE punishment_ladder_executions (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    guild_id BIGINT UNSIGNED NOT NULL,
    target_user_id BIGINT UNSIGNED NOT NULL,
    rule_id BIGINT UNSIGNED NULL,
    case_id BIGINT UNSIGNED NULL,
    warning_id BIGINT UNSIGNED NOT NULL,
    warning_count INT UNSIGNED NOT NULL,
    action VARCHAR(32) NOT NULL,
    status VARCHAR(16) NOT NULL,
    failure_reason TEXT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    completed_at TIMESTAMP(3) NULL,
    CONSTRAINT punishment_ladder_executions_rule_fk
        FOREIGN KEY (rule_id) REFERENCES punishment_ladder_rules (id) ON DELETE SET NULL,
    CONSTRAINT punishment_ladder_executions_case_fk
        FOREIGN KEY (case_id) REFERENCES moderation_cases (id) ON DELETE SET NULL,
    CONSTRAINT punishment_ladder_executions_warning_fk
        FOREIGN KEY (warning_id) REFERENCES moderation_warnings (id) ON DELETE CASCADE,
    UNIQUE KEY punishment_ladder_executions_warning (warning_id),
    KEY punishment_ladder_executions_target (guild_id, target_user_id, created_at, id)
);

CREATE TABLE channel_lock_operations (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    case_id BIGINT UNSIGNED NOT NULL,
    guild_id BIGINT UNSIGNED NOT NULL,
    actor_user_id BIGINT UNSIGNED NOT NULL,
    action VARCHAR(16) NOT NULL,
    reason TEXT NULL,
    status VARCHAR(16) NOT NULL,
    due_at TIMESTAMP(3) NULL,
    claimed_at TIMESTAMP(3) NULL,
    claimed_by VARCHAR(128) NULL,
    completed_at TIMESTAMP(3) NULL,
    failure_reason TEXT NULL,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    CONSTRAINT channel_lock_operations_case_fk
        FOREIGN KEY (case_id) REFERENCES moderation_cases (id) ON DELETE CASCADE,
    UNIQUE KEY channel_lock_operations_case (case_id),
    KEY channel_lock_operations_due (status, due_at, id),
    KEY channel_lock_operations_guild (guild_id, created_at, id)
);

CREATE TABLE channel_lock_subjects (
    guild_id BIGINT UNSIGNED NOT NULL,
    channel_id BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (guild_id, channel_id)
);

CREATE TABLE channel_lock_targets (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    operation_id BIGINT UNSIGNED NOT NULL,
    channel_id BIGINT UNSIGNED NOT NULL,
    overwrite_target_id BIGINT UNSIGNED NOT NULL,
    overwrite_target_kind VARCHAR(16) NOT NULL,
    previous_allow BIGINT UNSIGNED NULL,
    previous_deny BIGINT UNSIGNED NULL,
    status VARCHAR(16) NOT NULL,
    failure_reason TEXT NULL,
    completed_at TIMESTAMP(3) NULL,
    CONSTRAINT channel_lock_targets_operation_fk
        FOREIGN KEY (operation_id) REFERENCES channel_lock_operations (id) ON DELETE CASCADE,
    UNIQUE KEY channel_lock_targets_unique
        (operation_id, channel_id, overwrite_target_id, overwrite_target_kind),
    KEY channel_lock_targets_operation (operation_id, status)
);

-- One row per process boot, so reconnects/restarts cannot masquerade as a
-- continuously healthy bot. Snapshots contain diagnostics only, never secrets.
CREATE TABLE bot_runtime (
    instance_id VARCHAR(36) NOT NULL PRIMARY KEY,
    snapshot JSON NOT NULL,
    stopped BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    KEY bot_runtime_updated (updated_at)
);

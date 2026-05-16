-- scripts: persistent scheduled job records.
--
-- handler_key references a registered ScriptHandler at runtime. interval_ms
-- is how often the job repeats (0 means one-shot). next_run_at is unix
-- epoch milliseconds. state is per-job persistent JSON (serialized).
CREATE TABLE scripts (
    id           BIGINT  NOT NULL CHECK (id > 0),
    object_uid   BIGINT  NULL,
    handler_key  TEXT    NOT NULL,
    interval_ms  BIGINT  NOT NULL DEFAULT 0,
    next_run_at  BIGINT  NOT NULL,
    repeat       INTEGER NOT NULL DEFAULT 1,
    state        TEXT    NOT NULL DEFAULT '{}',
    enabled      INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (id),
    FOREIGN KEY (object_uid) REFERENCES objects(uid) ON DELETE CASCADE
);

CREATE INDEX idx_scripts_due ON scripts(enabled, next_run_at);
CREATE INDEX idx_scripts_handler ON scripts(handler_key);
CREATE INDEX idx_scripts_object ON scripts(object_uid);

-- objects: a generic in-world entity. type_key discriminates rooms, items,
-- characters, mobs, exits, etc. Downstream framework users register their
-- own type_keys; the framework treats them all uniformly.
CREATE TABLE objects (
    uid          BIGINT  NOT NULL CHECK (uid > 0),
    type_key     TEXT    NOT NULL,
    name         TEXT    NOT NULL,
    location_uid BIGINT  NULL,
    created_at   BIGINT  NOT NULL,
    PRIMARY KEY (uid),
    FOREIGN KEY (location_uid) REFERENCES objects(uid) ON DELETE SET NULL
);

CREATE INDEX idx_objects_type_key ON objects(type_key);
CREATE INDEX idx_objects_location ON objects(location_uid);

-- object_attributes: per-object freeform key/value bag. value is a JSON-encoded
-- string (use serde_json::Value at the Rust layer). Avoids schema migration
-- for downstream gameplay state.
CREATE TABLE object_attributes (
    object_uid BIGINT NOT NULL,
    key        TEXT   NOT NULL,
    value      TEXT   NOT NULL,
    PRIMARY KEY (object_uid, key),
    FOREIGN KEY (object_uid) REFERENCES objects(uid) ON DELETE CASCADE
);

-- object_tags: labels with optional category. Used for grouping and lookups
-- ("all rooms tagged 'safe-zone'", "all objects with the 'pvp' permission").
CREATE TABLE object_tags (
    object_uid BIGINT NOT NULL,
    key        TEXT   NOT NULL,
    category   TEXT   NOT NULL DEFAULT '',
    PRIMARY KEY (object_uid, key, category),
    FOREIGN KEY (object_uid) REFERENCES objects(uid) ON DELETE CASCADE
);

CREATE INDEX idx_object_tags_lookup ON object_tags(category, key);

-- character_accounts: link table between objects (where type_key='character')
-- and the owning account. Replaces the old account_characters/characters
-- pairing.
CREATE TABLE character_accounts (
    object_uid  BIGINT  NOT NULL,
    account_uid BIGINT  NOT NULL,
    ordinal     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (object_uid),
    FOREIGN KEY (object_uid) REFERENCES objects(uid) ON DELETE CASCADE,
    FOREIGN KEY (account_uid) REFERENCES accounts(uid) ON DELETE CASCADE
);

CREATE INDEX idx_character_accounts_account ON character_accounts(account_uid, ordinal);

-- Drop the now-redundant standalone tables. character data has migrated to
-- objects + character_accounts.
DROP TABLE IF EXISTS account_characters;
DROP TABLE IF EXISTS characters;

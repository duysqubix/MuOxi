CREATE TABLE accounts (
    uid           BIGINT       NOT NULL CHECK (uid > 0),
    name          VARCHAR(64)  NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    email         VARCHAR(255) NOT NULL DEFAULT '',
    created_at    BIGINT       NOT NULL,
    PRIMARY KEY (uid)
);

CREATE INDEX idx_accounts_name ON accounts(name);

CREATE TABLE characters (
    uid         BIGINT       NOT NULL CHECK (uid > 0),
    account_uid BIGINT       NOT NULL CHECK (account_uid > 0),
    name        VARCHAR(64)  NOT NULL UNIQUE,
    created_at  BIGINT       NOT NULL,
    PRIMARY KEY (uid),
    FOREIGN KEY (account_uid) REFERENCES accounts(uid) ON DELETE CASCADE
);

CREATE INDEX idx_characters_account ON characters(account_uid);

CREATE TABLE account_characters (
    account_uid   BIGINT  NOT NULL,
    character_uid BIGINT  NOT NULL,
    ordinal       INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_uid, character_uid),
    FOREIGN KEY (account_uid) REFERENCES accounts(uid) ON DELETE CASCADE,
    FOREIGN KEY (character_uid) REFERENCES characters(uid) ON DELETE CASCADE
);

CREATE INDEX idx_account_characters_ordinal
    ON account_characters(account_uid, ordinal);

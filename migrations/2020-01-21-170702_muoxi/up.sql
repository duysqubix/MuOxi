-- Your SQL goes here
CREATE TABLE Accounts (
    uid BIGINT NOT NULL CHECK (uid > 0),
    name varchar(64) NOT NULL,
    password varchar(64) NOT NULL,
    email varchar(64) NOT NULL,
    characters BIGINT ARRAY,
    PRIMARY KEY (uid)
);

CREATE TABLE Characters (
    uid BIGINT NOT NULL CHECK (uid > 0),
    account BIGINT NOT NULL CHECK (account > 0),
    name varchar(64) NOT NULL,
    PRIMARY KEY (uid)
);


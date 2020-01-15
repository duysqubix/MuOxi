-- Your SQL goes here
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE Accounts (
    uid BIGINT NOT NULL CHECK (uid > 0),
    name varchar(64) NOT NULL,
    password varchar(64) NOT NULL,
    email varchar(64) NOT NULL,
    characters BIGINT ARRAY,
    PRIMARY KEY (uid))

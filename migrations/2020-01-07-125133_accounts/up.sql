-- Your SQL goes here
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE Clients(
    uid BIGINT NOT NULL CHECK (uid > 0),
    ip varchar(15) NOT NULL,
    port INTEGER NOT NULL,
    PRIMARY KEY (uid)
)
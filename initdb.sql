CREATE TYPE CREDENTIAL AS (
    username VARCHAR,
    password VARCHAR
);

CREATE TABLE IF NOT EXISTS networks (
    id UUID PRIMARY KEY,
    network VARCHAR NOT NULL,
    available INTEGER NOT NULL,
    used INTEGER NOT NULL,
    total INTEGER NOT NULL,
    vlan INTEGER,
    description VARCHAR
);

CREATE TABLE IF NOT EXISTS offices (
    id UUID PRIMARY KEY,
    description VARCHAR,
    address VARCHAR UNIQUE
);

CREATE TABLE IF NOT EXISTS devices (
    ip VARCHAR NOT NULL,
    description VARCHAR,
    office_id UUID,
    rack VARCHAR,
    room VARCHAR,
    status VARCHAR NOT NULL,
    network_id UUID NOT NULL,
    credential CREDENTIAL,
    PRIMARY KEY (ip, network_id),
    FOREIGN KEY (network_id) REFERENCES networks(id) ON DELETE CASCADE,
    FOREIGN KEY (office_id) REFERENCES offices(id) ON DELETE SET NULL
);

CREATE TYPE ROLE AS ENUM ('Admin', 'Operator', 'Guest');

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR UNIQUE,
    password TEXT,
    role ROLE
);
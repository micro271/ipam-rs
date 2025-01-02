CREATE TYPE STATUS as ENUM ('Reserved', 'Unknown', 'Online', 'Offline');

CREATE TABLE IF NOT EXISTS networks (
    id UUID PRIMARY KEY,
    network VARCHAR NOT NULL,
    available INTEGER NOT NULL,
    used INTEGER NOT NULL,
    free INTEGER NOT NULL,
    vlan INTEGER,
    description VARCHAR,
    father UUID,
    FOREIGN KEY father REFERENCES networks(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS devices (
    ip VARCHAR NOT NULL,
    description VARCHAR,
    label TEXT,
    room UUID,
    mount_point TEXT,
    status STATUS NOT NULL,
    network_id UUID NOT NULL,
    username TEXT,
    password TEXT,
    PRIMARY KEY (ip, network_id),
    FOREIGN KEY (network_id) REFERENCES networks(id) ON DELETE CASCADE,
    FOREIGN KEY (label, room, mount_point) REFERENCES locations(label, role, mount_point) ON DELETE SET NULL ON UPDATE SET NULL
);

CREATE TABLE IF NOT EXISTS mount_point (
    name TEXT,
    PRIMARY KEY (name)
);

CREATE TABLE IF NOT EXISTS locations (
    label TEXT,
    mount_point TEXT,
    id_room TEXT,
    address TEXT NOT NULL,
    PRIMARY KEY (label, mount_point, id_room),
    FOREIGN KEY (mount_point) REFERENCES mount_point(name) ON DELETE SET NULL ON UPDATE CASCADE,
    FOREIGN KEY (id_room, address) REFERENCES room(id, address) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS offices (
    description VARCHAR,
    address TEXT,
    PRIMARY KEY (address)
);

CREATE TABLE IF NOT EXISTS room (
    id TEXT,
    address TEXT,
    PRIMARY KEY (id, address),
    FOREIGN KEY (address) REFERENCES offices(address) ON DELETE CASCADE
);

CREATE TYPE ROLE AS ENUM ('Admin', 'Operator', 'Guest');

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(32) UNIQUE,
    password TEXT,
    role ROLE
);
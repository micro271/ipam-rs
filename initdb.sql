CREATE TYPE STATUS as ENUM ('Reserved', 'Unknown', 'Online', 'Offline');

CREATE TABLE IF NOT EXISTS vlans (
    id INTEGER,
    description TEXT,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS networks (
    id UUID PRIMARY KEY,
    network VARCHAR NOT NULL,
    available INTEGER NOT NULL,
    used INTEGER NOT NULL,
    free INTEGER NOT NULL,
    vlan INTEGER,
    description VARCHAR,
    father UUID,
    children INTEGER,
    FOREIGN KEY (father) REFERENCES networks(id) ON DELETE CASCADE,
    FOREIGN KEY (vlan) REFERENCES vlans(id) ON DELETE SET NULL ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS mount_point (
    name TEXT,
    PRIMARY KEY (name)
);

CREATE TABLE IF NOT EXISTS offices (
    description VARCHAR,
    address TEXT,
    PRIMARY KEY (address)
);

CREATE TABLE IF NOT EXISTS room (
    name TEXT,
    address TEXT,
    PRIMARY KEY (name, address),
    FOREIGN KEY (address) REFERENCES offices(address) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS locations (
    label TEXT,
    mount_point TEXT,
    room_name TEXT,
    address TEXT NOT NULL,
    PRIMARY KEY (label, mount_point, room_name),
    FOREIGN KEY (mount_point) REFERENCES mount_point(name) ON DELETE SET NULL ON UPDATE CASCADE,
    FOREIGN KEY (room_name, address) REFERENCES room(name, address) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS devices (
    ip VARCHAR NOT NULL,
    description VARCHAR,
    label TEXT,
    room_name TEXT,
    mount_point TEXT,
    status STATUS NOT NULL,
    network_id UUID NOT NULL,
    username TEXT,
    password TEXT,
    PRIMARY KEY (ip, network_id),
    FOREIGN KEY (network_id) REFERENCES networks(id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (label, room_name, mount_point) REFERENCES locations(label, room_name, mount_point) ON DELETE SET NULL ON UPDATE SET NULL
);


CREATE TYPE ROLE AS ENUM ('Admin', 'Operator', 'Guest');

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(32) UNIQUE,
    password TEXT,
    role ROLE
);
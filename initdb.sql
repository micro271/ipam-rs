CREATE TYPE STATUSADDR as ENUM ('Reserved', 'Unknown', 'Online', 'Offline', 'Reachable');
CREATE TYPE ROLE AS ENUM ('Admin', 'Operator', 'Guest');
CREATE TYPE STATUS_NETWORK AS ENUM ('Available', 'Used', 'Reserved');
CREATE TYPE KIND_NETWORK AS ENUM ('Network', 'Pool');

CREATE TABLE IF NOT EXISTS vlans (
    id INTEGER,
    description TEXT,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS networks (
    id UUID PRIMARY KEY,
    subnet TEXT NOT NULL,
    used INTEGER NOT NULL,
    free INTEGER NOT NULL,
    vlan INTEGER,
    description VARCHAR,
    father UUID,
    children INTEGER,
    status STATUS_NETWORK,
    kind KIND_NETWORK,
    node UUID,
    FOREIGN KEY (father) REFERENCES networks(id) ON DELETE CASCADE,
    FOREIGN KEY (vlan) REFERENCES vlans(id) ON DELETE SET NULL ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS addresses (
    ip TEXT,
    network_id UUID,
    status STATUSADDR,
    node_id UUID,
    PRIMARY KEY (ip, network_id),
    FOREIGN KEY (network_id) REFERENCES networks (id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS nodes (
    id UUID,
    hostname TEXT,
    description TEXT,
    username TEXT,
    password TEXT,
    PRIMARY KEY (id),
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(32) UNIQUE,
    password TEXT,
    role ROLE,
    is_active BOOLEAN DEFAULT TRUE,
    create_at TIMESTAMPTZ,
    last_login TIMESTAMPTZ
);
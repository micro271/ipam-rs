CREATE TYPE STATUS as ENUM ('Reserved', 'Unknown', 'Online', 'Offline');
CREATE TYPE ROLE AS ENUM ('Admin', 'Operator', 'Guest');
CREATE TYPE TARGET AS ENUM ('Node', 'Nat');
CREATE TYPE STATUS_NETWORK AS ENUM ('Available', 'Used', 'Reserved');

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
    status STATUS_NETWORK,
    target TARGET,
    FOREIGN KEY (father) REFERENCES networks(id) ON DELETE CASCADE,
    FOREIGN KEY (vlan) REFERENCES vlans(id) ON DELETE SET NULL ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS mount_point (
    name TEXT,
    PRIMARY KEY (name)
);

CREATE TABLE IF NOT EXISTS offices (
    id UUID,
    description TEXT,
    street TEXT,
    neighborhood TEXT,
    UNIQUE (neighborhood, street),
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS room (
    name TEXT,
    id_office UUID,
    PRIMARY KEY (name, id_office),
    FOREIGN KEY (id_office) REFERENCES offices(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS locations (
    label TEXT,
    mount_point TEXT,
    room_name TEXT,
    id_office UUID NOT NULL,
    PRIMARY KEY (label, mount_point, room_name),
    FOREIGN KEY (mount_point) REFERENCES mount_point(name) ON DELETE SET NULL ON UPDATE CASCADE,
    FOREIGN KEY (room_name, id_office) REFERENCES room(name, id_office) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS nodes (
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

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(32) UNIQUE,
    password TEXT,
    role ROLE,
    is_active BOOLEAN DEFAULT TRUE,
    create_at TIMESTAMPTZ,
    last_login TIMESTAMPTZ
);
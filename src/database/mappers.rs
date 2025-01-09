use crate::models::{
    device::Device, location::Location, mound_point::MountPoint, network::Network, office::Office,
    room::Room, user::User,
};
use sqlx::{postgres::PgRow, Row};

impl From<PgRow> for Network {
    fn from(value: PgRow) -> Self {
        Self {
            id: value.get("id"),
            description: value.get("description"),
            network: value.get::<'_, &str, _>("network").parse().unwrap(),
            available: value.get("available"),
            used: value.get("available"),
            free: value.get("available"),
            vlan: value.get("vlan"),
            father: value.get("father"),
            children: value.get("children"),
        }
    }
}

impl From<PgRow> for Device {
    fn from(value: PgRow) -> Self {
        Self {
            ip: value.get::<'_, &str, _>("ip").parse().unwrap(),
            description: value.get("description"),
            room: value.get("room"),
            label: value.get("label"),
            mount_point: value.get("mount_point"),
            username: value.get("username"),
            password: value.get("password"),
            status: value.get("status"),
            network_id: value.get("network_id"),
        }
    }
}

impl From<PgRow> for Office {
    fn from(value: PgRow) -> Self {
        Self {
            id: value.get("id"),
            name: value.get("description"),
            address: value.get("address"),
            description: value.get("description"),
        }
    }
}

impl From<PgRow> for User {
    fn from(value: PgRow) -> Self {
        Self {
            id: value.get("id"),
            username: value.get("username"),
            password: value.get("password"),
            role: value.get("role"),
        }
    }
}

impl From<PgRow> for Location {
    fn from(value: PgRow) -> Self {
        Self {
            label: value.get("label"),
            mont_point: value.get("mount_point"),
            id_room: value.get("id_room"),
            address: value.get("address"),
        }
    }
}

impl From<PgRow> for MountPoint {
    fn from(value: PgRow) -> Self {
        Self {
            name: value.get("name"),
        }
    }
}

impl From<PgRow> for Room {
    fn from(value: PgRow) -> Self {
        Self {
            id: value.get("id"),
            address: value.get("address"),
        }
    }
}

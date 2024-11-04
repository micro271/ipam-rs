use crate::models::{
    office::Office,
    {
        device::{Credential, Device},
        network::{Network, Vlan},
        user::User,
    },
};
use sqlx::{postgres::PgRow, Row};
use uuid::Uuid;

impl From<PgRow> for Network {
    fn from(value: PgRow) -> Self {
        Self {
            id: value.get("id"),
            description: value.get("description"),
            network: value.get::<'_, &str, _>("network").parse().unwrap(),
            available: value.get::<'_, i32, &str>("available") as u32,
            used: value.get::<'_, i32, _>("used") as u32,
            total: value.get::<'_, i32, _>("total") as u32,
            vlan: Some(Vlan::new(value.get::<'_, i32, _>("vlan")).unwrap()),
        }
    }
}

impl From<PgRow> for Device {
    fn from(value: PgRow) -> Self {
        Self {
            ip: value.get::<'_, &str, _>("ip").parse().unwrap(),
            description: value.get("description"),
            office_id: value.get("office_id"),
            rack: value.get("rack"),
            credential: {
                let cred: Option<(String, String)> = value.get("credential");
                cred.map(|(username, password)| Credential { username, password })
            },
            room: value.get("room"),
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

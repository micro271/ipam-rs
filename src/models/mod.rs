pub mod utils;

use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Row};
use std::{collections::HashMap, net::IpAddr};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateDevice {
    pub ip: Option<IpAddr>,
    pub description: Option<String>,
    pub office_id: Option<Uuid>,
    pub rack: Option<Option<String>>,
    pub room: Option<Option<String>>,
    pub status: Option<Status>,
    pub network_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateNetwork {
    pub network: Option<IpNet>,
    pub description: Option<String>,
    pub available: Option<u32>,
    pub used: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Device {
    pub ip: IpAddr,
    pub description: Option<String>,
    pub office_id: Option<Uuid>,
    pub rack: Option<String>,
    pub room: Option<String>,
    pub status: Status,
    pub network_id: uuid::Uuid,
    pub credential: Option<Credential>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Credential {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub id: uuid::Uuid,
    pub vlan: Option<Vlan>,
    pub network: IpNet,
    pub description: Option<String>,
    pub available: u32,
    pub used: u32,
    pub total: u32,
}

#[derive(Debug)]
pub struct Vlan(pub u16);

#[derive(Debug, Deserialize, Serialize)]
pub struct Office {
    pub id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub description: Option<String>,
}

pub struct UpdateOffice {
    description: Option<String>,
    address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type, PartialEq)]
pub enum Status {
    Reserved,
    Unknown,
    Online,
    Offline,
}

impl Default for Status {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<PgRow> for Device {
    fn from(value: PgRow) -> Self {
        Self {
            ip: value.get::<'_, &str, _>("ip").parse().unwrap(),
            description: value.get("description"),
            office_id: value.get("office_ids"),
            rack: value.get("rack"),
            credential: {
                let cred: Option<(String, String)> = value.get("credential");
                cred.map(|(username, password)| Credential { username, password })
            },
            room: value.get("room"),
            status: value.get("status"),
            network_id: Uuid::parse_str(value.get("network_status")).unwrap(),
        }
    }
}

impl From<PgRow> for Network {
    fn from(value: PgRow) -> Self {
        Self {
            id: value.get("id"),
            description: value.get("description"),
            network: value.get::<'_, &str, _>("network").parse().unwrap(),
            available: value.get::<'_, i32, &str>("available") as u32,
            used: value.get::<'_, i32, _>("used") as u32,
            total: value.get::<'_, i32, _>("total") as u32,
            vlan: Some(Vlan(value.get::<'_, i32, _>("vlan") as u16)),
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

impl Serialize for Vlan {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.0)
    }
}

impl<'de> Deserialize<'de> for Vlan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(u16::deserialize(deserializer)?))
    }
}

impl std::ops::Deref for Vlan {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Vlan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

use super::*;
use ipnet::IpNet;
use std::{net::IpAddr, vec};
use uuid::Uuid;

pub trait Table {
    fn name() -> String;
    fn query_insert() -> String;
    fn get_fields(self) -> Vec<TypeTable>;
    fn columns() -> Vec<&'static str>;
}

pub trait Updatable<'a> {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>>;
}

impl Table for Device {
    fn columns() -> Vec<&'static str> {
        vec![
            "ip",
            "description",
            "office",
            "rack",
            "room",
            "status",
            "network_id",
        ]
    }

    fn name() -> String {
        String::from("devices")
    }

    fn query_insert() -> String {
        format!("INSERT INTO {} (ip, network_id, description, office, rack, room, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)", Self::name())
    }

    fn get_fields(self) -> Vec<TypeTable> {
        vec![
            self.ip.into(),
            self.network_id.into(),
            self.description.into(),
            self.office_id.into(),
            self.rack.into(),
            self.room.into(),
            self.status.into(),
        ]
    }
}

impl Table for Network {
    fn columns() -> Vec<&'static str> {
        vec!["id", "network", "description", "available", "used", "total"]
    }

    fn name() -> String {
        String::from("network")
    }

    fn query_insert() -> String {
        format!(
            "INSERT INTO {} (network, available, used, total, vlan) VALUES ($1, $2, $3, $4, &5)",
            Self::name()
        )
    }

    fn get_fields(self) -> Vec<TypeTable> {
        vec![
            self.network.into(),
            self.available.into(),
            self.used.into(),
            self.total.into(),
            self.vlan.into(),
        ]
    }
}

impl Table for Office {
    fn name() -> String {
        String::from("offices")
    }

    fn query_insert() -> String {
        format!(
            "INSERT INTO {} (id, description, address) VALUES ($1, $2, $3)",
            Office::name()
        )
    }

    fn get_fields(self) -> Vec<TypeTable> {
        vec![self.id.into(), self.description.into(), self.address.into()]
    }

    fn columns() -> Vec<&'static str> {
        todo!()
    }
}

impl<'a> Updatable<'a> for UpdateDevice {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
        let mut pair = HashMap::new();

        if let Some(tmp) = self.ip {
            pair.insert("ip", tmp.into());
        }

        if let Some(tmp) = self.description {
            pair.insert("description", tmp.into());
        }

        if let Some(tmp) = self.network_id {
            pair.insert("network_id", tmp.into());
        }

        if let Some(tmp) = self.office_id {
            pair.insert("office", tmp.into());
        }

        if let Some(tmp) = self.rack {
            pair.insert("rack", tmp.into());
        }

        if let Some(tmp) = self.room {
            pair.insert("room", tmp.into());
        }

        if let Some(tmp) = self.status {
            pair.insert("status", tmp.into());
        }

        if !pair.is_empty() {
            Some(pair)
        } else {
            None
        }
    }
}

impl<'a> Updatable<'a> for UpdateNetwork {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
        let mut pair = HashMap::new();

        if let Some(tmp) = self.description {
            pair.insert("description", tmp.into());
        }

        if let Some(tmp) = self.network {
            pair.insert("network", tmp.into());
        }

        if let Some(tmp) = self.available {
            pair.insert("available", tmp.into());
        }

        if let Some(tmp) = self.used {
            pair.insert("used", tmp.into());
        }

        if let Some(tmp) = self.total {
            pair.insert("total", tmp.into());
        }

        if !pair.is_empty() {
            Some(pair)
        } else {
            None
        }
    }
}

impl<'a> Updatable<'a> for UpdateOffice {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
        let mut resp = HashMap::new();
        if let Some(tmp) = self.address {
            resp.insert("address", tmp.into());
        }

        if let Some(tmp) = self.description {
            resp.insert("description", tmp.into());
        }

        Some(resp)
    }
}

pub enum TypeTable {
    String(String),
    OptionUuid(Option<Uuid>),
    Uuid(Uuid),
    OptionString(Option<String>),
    Status(super::Status),
    Int32(i32),
    Role(crate::user::Role),
    Float64(f64),
    OptionVlan(Option<i32>),
}

impl From<Option<Vlan>> for TypeTable {
    fn from(value: Option<Vlan>) -> Self {
        Self::OptionVlan(value.map(|vlan| *vlan as i32))
    }
}

impl From<Uuid> for TypeTable {
    fn from(value: Uuid) -> Self {
        TypeTable::Uuid(value)
    }
}

impl From<crate::user::Role> for TypeTable {
    fn from(value: crate::user::Role) -> Self {
        Self::Role(value)
    }
}

// impl From<Uuid> for TypeTable {
//     fn from(value: Uuid) -> Self {
//         Self::String(value.to_string())
//     }
// }

impl From<Option<Uuid>> for TypeTable {
    fn from(value: Option<Uuid>) -> Self {
        Self::OptionUuid(value)
    }
}

impl From<IpAddr> for TypeTable {
    fn from(value: IpAddr) -> Self {
        Self::String(value.to_string())
    }
}

impl From<IpNet> for TypeTable {
    fn from(value: IpNet) -> Self {
        Self::String(value.to_string())
    }
}

impl From<u8> for TypeTable {
    fn from(value: u8) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<u16> for TypeTable {
    fn from(value: u16) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<u32> for TypeTable {
    fn from(value: u32) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<i8> for TypeTable {
    fn from(value: i8) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<i16> for TypeTable {
    fn from(value: i16) -> Self {
        Self::Int32(value as i32)
    }
}

impl From<i32> for TypeTable {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<String> for TypeTable {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Option<String>> for TypeTable {
    fn from(value: Option<String>) -> Self {
        Self::OptionString(value)
    }
}

impl From<super::Status> for TypeTable {
    fn from(value: super::Status) -> Self {
        Self::Status(value)
    }
}

impl From<f32> for TypeTable {
    fn from(value: f32) -> Self {
        Self::Float64(value as f64)
    }
}

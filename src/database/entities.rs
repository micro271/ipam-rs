use super::HashMap;
use super::{Table, TypeTable, Updatable};
use crate::models::{device::*, network::*, office::*, user::*};
use std::net::IpAddr;

impl Table for User {
    fn columns() -> Vec<&'static str> {
        vec!["id", "username", "password", "role"]
    }
    fn name() -> String {
        String::from("USERS")
    }

    fn get_fields(self) -> Vec<TypeTable> {
        vec![
            self.id.into(),
            self.username.into(),
            self.password.into(),
            self.role.into(),
        ]
    }
}

impl Table for Device {
    fn columns() -> Vec<&'static str> {
        vec![
            "ip",
            "description",
            "office_id",
            "rack",
            "room",
            "status",
            "network_id",
            "credential",
        ]
    }

    fn name() -> String {
        String::from("devices")
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
            self.credential.into(),
        ]
    }
}

impl Table for Network {
    fn columns() -> Vec<&'static str> {
        vec![
            "id",
            "network",
            "description",
            "available",
            "used",
            "total",
            "vlan",
        ]
    }

    fn name() -> String {
        String::from("networks")
    }

    fn get_fields(self) -> Vec<TypeTable> {
        vec![
            self.id.into(),
            self.network.into(),
            self.available.into(),
            self.used.into(),
            self.free.into(),
            self.vlan.into(),
            self.description.into(),
        ]
    }
}

impl Table for Office {
    fn name() -> String {
        String::from("offices")
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
            let data = if tmp.is_empty() { Some(tmp) } else { None };
            pair.insert("description", data.into());
        }

        if let Some(tmp) = self.network_id {
            pair.insert("network_id", tmp.into());
        }

        if let Some(tmp) = self.office_id {
            let data = if tmp == uuid::Uuid::nil() {
                None
            } else {
                Some(tmp)
            };
            pair.insert("office", data.into());
        }

        if let Some(tmp) = self.rack {
            let data = if tmp.is_empty() { None } else { Some(tmp) };
            pair.insert("rack", data.into());
        }

        if let Some(tmp) = self.room {
            let data = if tmp.is_empty() { None } else { Some(tmp) };
            pair.insert("room", data.into());
        }

        if let Some(tmp) = self.status {
            pair.insert("status", tmp.into());
        }

        if let Some(cred) = self.credential {
            let data = if cred.password.is_empty() && cred.username.is_empty() {
                None
            } else {
                Some(cred)
            };
            pair.insert("credential", data.into());
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
            let data = if tmp.is_empty() { None } else { Some(tmp) };
            pair.insert("description", data.into());
        }

        if let Some(tmp) = self.network {
            pair.insert("network", tmp.into());
        }

        if let Some(vlan) = self.vlan {
            let data = if *vlan == 0 { None } else { Some(vlan) };

            pair.insert("vlan", data.into());
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

impl<'a> Updatable<'a> for IpAddr {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
        Some(HashMap::from([("ip", self.into())]))
    }
}

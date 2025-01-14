use super::HashMap;
use super::{Table, TypeTable, Updatable};
use crate::models::location::{Location, LocationUpdate};
use crate::models::{device::*, network::*, office::*, user::*};
use std::net::IpAddr;

impl Table for Location {
    fn name() -> String {
        String::from("locations")
    }

    fn get_fields(self) -> Vec<TypeTable> {
        vec![
            self.mont_point.into(),
            self.room_name.into(),
            self.label.into(),
            self.address.into(),
        ]
    }

    fn columns() -> Vec<&'static str> {
        vec!["mount_point", "room_name", "label", "address"]
    }
}

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
            "label",
            "room",
            "mount_point",
            "status",
            "network_id",
            "username",
            "password",
        ]
    }

    fn name() -> String {
        String::from("devices")
    }

    fn get_fields(self) -> Vec<TypeTable> {
        vec![
            self.ip.into(),
            self.description.into(),
            self.label.into(),
            self.room.into(),
            self.mount_point.into(),
            self.status.into(),
            self.network_id.into(),
            self.username.into(),
            self.password.into(),
        ]
    }
}

impl Table for Network {
    fn columns() -> Vec<&'static str> {
        vec![
            "id",
            "network",
            "available",
            "used",
            "free",
            "vlan",
            "description",
            "farther",
            "children",
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
            self.father.into(),
            self.children.into(),
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

impl<'a> Updatable<'a> for UpdateUser {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
        let mut resp = HashMap::new();
        if let Some(e) = self.username {
            resp.insert("username", e.into());
        }

        if let Some(e) = self.password {
            resp.insert(
                "password",
                match libipam::authentication::encrypt(e) {
                    Ok(e) => e,
                    Err(_) => return None,
                }
                .into(),
            );
        }

        if let Some(e) = self.role {
            resp.insert("role", e.into());
        }

        if resp.is_empty() {
            None
        } else {
            Some(resp)
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

impl<'a> Updatable<'a> for HashMap<&'a str, TypeTable> {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
        Some(self)
    }
}

impl<'a> Updatable<'a> for IpAddr {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
        Some(HashMap::from([("ip", self.into())]))
    }
}

impl<'a> Updatable<'a> for LocationUpdate {
    fn get_pair(self) -> Option<HashMap<&'a str, TypeTable>> {
        let mut cond = HashMap::new();

        if let Some(tmp) = self.room_name {
            cond.insert(
                "room_name",
                if tmp.is_empty() { tmp } else { return None }.into(),
            );
        }

        if let Some(tmp) = self.address {
            cond.insert(
                "address",
                if tmp.is_empty() { tmp } else { return None }.into(),
            );
        }

        if let Some(tmp) = self.label {
            cond.insert(
                "label",
                if tmp.is_empty() { tmp } else { return None }.into(),
            );
        }

        if let Some(tmp) = self.mont_point {
            cond.insert(
                "mont_point",
                if tmp.is_empty() { tmp } else { return None }.into(),
            );
        }

        Some(cond)
    }
}

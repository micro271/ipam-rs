use serde::Deserialize;
use std::{collections::HashMap, net::IpAddr};

use crate::database::repository::TypeTable;

#[derive(Debug, Deserialize)]
pub struct ParamsDevice {
    pub ip: Option<IpAddr>,
    pub network_id: Option<uuid::Uuid>,
}

impl GetMapParams for ParamsDevice {
    fn get_pairs(self) -> Option<HashMap<&'static str, TypeTable>> {
        let mut resp = HashMap::new();

        if let Some(e) = self.ip {
            resp.insert("ip", e.into());
        }
        if let Some(e) = self.network_id {
            resp.insert("network_id", e.into());
        }

        if resp.is_empty() {
            None
        } else {
            Some(resp)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ParamsDeviceStrict {
    pub ip: IpAddr,
    pub network_id: uuid::Uuid,
}

impl GetMapParams for ParamsDeviceStrict {
    fn get_pairs(self) -> Option<HashMap<&'static str, TypeTable>> {
        Some(HashMap::from([
            ("ip", self.ip.into()),
            ("network_id", self.network_id.into()),
        ]))
    }
}

#[derive(Deserialize, Debug)]
pub struct Subnet {
    pub father: uuid::Uuid,
    pub prefix: u8,
}

pub trait GetMapParams {
    fn get_pairs(self) -> Option<HashMap<&'static str, TypeTable>>;
}

#[derive(Debug, Deserialize)]
pub struct LocationParam {
    pub label: Option<String>,
    pub room_name: Option<String>,
    pub mount_point: Option<String>,
}

impl GetMapParams for LocationParam {
    fn get_pairs(self) -> Option<HashMap<&'static str, TypeTable>> {
        let mut condition = HashMap::new();
        if let Some(label) = self.label.filter(|x| x.is_empty()) {
            condition.insert("label", label.into());
        }

        if let Some(mount_point) = self.mount_point.filter(|x| x.is_empty()) {
            condition.insert("mount_point", mount_point.into());
        }

        if let Some(room_name) = self.room_name.filter(|x| x.is_empty()) {
            condition.insert("room_name", room_name.into());
        }

        Some(condition)
    }
}

#[derive(Debug, Deserialize)]
pub struct LocationParamStict {
    pub label: String,
    pub room_name: String,
    pub mount_point: String,
}

impl GetMapParams for LocationParamStict {
    fn get_pairs(self) -> Option<HashMap<&'static str, TypeTable>> {
        Some(HashMap::from([
            ("label", self.label.into()),
            ("room_name", self.room_name.into()),
            ("mount_point", self.mount_point.into()),
        ]))
    }
}

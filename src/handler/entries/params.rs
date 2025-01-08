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

pub struct Subnet {
    pub father: uuid::Uuid,
    pub prefix: u8,
}

pub trait GetMapParams {
    fn get_pairs(self) -> Option<HashMap<&'static str, TypeTable>>;
}

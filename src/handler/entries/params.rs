use crate::database::repository::TypeTable;
use ipnet::IpNet;
use serde::Deserialize;
use std::{collections::HashMap, net::IpAddr};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ParamsDevice {
    pub ip: IpAddr,
    pub network_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct QueryNetwork {
    pub id: Option<Uuid>,
    pub description: Option<String>,
    pub network: Option<IpNet>,
}

impl QueryNetwork {
    pub fn get_condition(self) -> Option<HashMap<&'static str, TypeTable>> {
        let mut resp: HashMap<&str, TypeTable> = HashMap::new();

        if let Some(id) = self.id {
            resp.insert("id", id.into());
        }
        if let Some(description) = self.description {
            resp.insert("id", description.into());
        }
        if let Some(network) = self.network {
            resp.insert("id", network.into());
        }

        if resp.is_empty() {
            None
        } else {
            Some(resp)
        }
    }
}

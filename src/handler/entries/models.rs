use super::super::models::{device, network};
use ipnet::IpNet;
use libipam::type_net::vlan::Vlan;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Network {
    pub network: IpNet,
    pub description: Option<String>,
    pub vlan: Option<Vlan>,
}

impl From<Network> for network::Network {
    fn from(value: Network) -> Self {
        let avl = 2_u32.pow(32 - value.network.prefix_len() as u32) - 2;
        Self {
            id: Uuid::new_v4(),
            network: value.network,
            description: value.description,
            available: avl.into(),
            used: 0.into(),
            free: avl.into(),
            vlan: value.vlan,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Device {
    pub ip: IpAddr,
    pub description: Option<String>,
    pub office_id: Option<Uuid>,
    pub rack: Option<String>,
    pub room: Option<String>,
    pub status: Option<device::Status>,
    pub network_id: uuid::Uuid,
    pub credential: Option<device::Credential>,
}

impl From<Device> for device::Device {
    fn from(value: Device) -> Self {
        Self {
            status: device::Status::default(),
            ip: value.ip,
            description: value.description,
            office_id: value.office_id,
            rack: value.rack,
            room: value.room,
            network_id: value.network_id,
            credential: value.credential,
        }
    }
}

pub fn create_all_devices(network: IpNet, id: Uuid) -> Option<Vec<device::Device>> {
    let ips = network.hosts().collect::<Vec<IpAddr>>();
    let mut resp = Vec::new();
    for ip in ips {
        resp.push(device::Device {
            ip,
            description: None,
            office_id: None,
            rack: None,
            room: None,
            status: device::Status::default(),
            network_id: id,
            credential: None,
        });
    }

    if !resp.is_empty() {
        Some(resp)
    } else {
        None
    }
}

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
            father: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Device {
    pub ip: IpAddr,
    pub description: Option<String>,
    pub label: Option<String>,
    pub room: Option<Uuid>,
    pub mount_point: Option<String>,
    pub network_id: uuid::Uuid,
    pub username: Option<String>,
    pub pasword: Option<String>,
}

impl From<Device> for device::Device {
    fn from(value: Device) -> Self {
        Self {
            ip: value.ip,
            description: value.description,
            room: value.room,
            label: value.label,
            mount_point: value.mount_point,
            status: device::Status::default(),
            network_id: value.network_id,
            username: value.username,
            password: value.pasword,
        }
    }
}

pub fn create_all_devices(network: IpNet, id: Uuid) -> Result<Vec<device::Device>, &'static str> {
    if network.network().is_ipv6() {
        return Err("You cannot create all devices of an network ipv6");
    }

    let ips = network.hosts().collect::<Vec<IpAddr>>();

    if ips.is_empty() {
        return Err("No devices created");
    }

    Ok(ips
        .into_iter()
        .map(|ip| device::Device {
            ip,
            description: None,
            mount_point: None,
            label: None,
            room: None,
            status: device::Status::default(),
            network_id: id,
            password: None,
            username: None,
        })
        .collect())
}

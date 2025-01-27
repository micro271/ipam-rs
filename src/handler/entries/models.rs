use crate::models::network::To;

use super::super::models::{
    device::{Device, Status},
    network::Network,
};
use ipnet::IpNet;
use libipam::type_net::vlan::VlanId;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkCreateEntry {
    pub network: IpNet,
    pub description: Option<String>,
    pub vlan: Option<VlanId>,
    pub to: Option<To>,
}

impl From<NetworkCreateEntry> for Network {
    fn from(value: NetworkCreateEntry) -> Self {
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
            children: 0,
            to: value.to.unwrap_or_default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeviceCreateEntry {
    pub ip: IpAddr,
    pub description: Option<String>,
    pub label: Option<String>,
    pub room: Option<Uuid>,
    pub mount_point: Option<String>,
    pub network_id: uuid::Uuid,
    pub username: Option<String>,
    pub pasword: Option<String>,
}

impl From<DeviceCreateEntry> for Device {
    fn from(value: DeviceCreateEntry) -> Self {
        Self {
            ip: value.ip,
            description: value.description,
            room: value.room,
            label: value.label,
            mount_point: value.mount_point,
            status: Status::default(),
            network_id: value.network_id,
            username: value.username,
            password: value.pasword,
        }
    }
}

pub fn create_all_devices(network: IpNet, id: Uuid) -> Result<Vec<Device>, &'static str> {
    if network.network().is_ipv6() {
        return Err("You cannot create all devices of an network ipv6");
    }

    let ips = network.hosts().collect::<Vec<IpAddr>>();

    if ips.is_empty() {
        return Err("The network doesn't have devices");
    }

    Ok(ips
        .into_iter()
        .map(|ip| Device {
            ip,
            description: None,
            mount_point: None,
            label: None,
            room: None,
            status: Status::default(),
            network_id: id,
            password: None,
            username: None,
        })
        .collect())
}

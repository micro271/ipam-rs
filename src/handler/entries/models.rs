use super::super::models::{
    device::{Device, Status},
    network::Network,
    user::User,
};
use crate::models::network::To;
use ipnet::IpNet;
use libipam::type_net::vlan::VlanId;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserEntry {
    pub username: String,
    pub password: String,
}

impl From<UserEntry> for User {
    fn from(value: UserEntry) -> Self {
        User {
            id: Uuid::new_v4(),
            username: value.username,
            password: value.password,
            role: crate::models::user::Role::Operator,
            is_active: true,
            create_at: time::OffsetDateTime::now_utc(),
            last_login: None,
        }
    }
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

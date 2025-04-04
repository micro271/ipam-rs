use super::super::models::{network::Network, node::Node, user::User};
use crate::models::network::{
    Kind, StatusNetwork,
    addresses::{Addresses, StatusAddr},
};
use ipnet::IpNet;
use libipam::types::vlan::VlanId;
use serde::{Deserialize, Serialize};
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
    pub subnet: IpNet,
    pub description: Option<String>,
    pub vlan: Option<VlanId>,
    pub kind: Option<Kind>,
}

impl From<NetworkCreateEntry> for Network {
    fn from(value: NetworkCreateEntry) -> Self {
        Self {
            id: Uuid::new_v4(),
            subnet: value.subnet,
            description: value.description,
            used: 0.try_into().unwrap(),
            free: value.subnet.into(),
            vlan: value.vlan,
            father: None,
            children: 0,
            status: StatusNetwork::default(),
            kind: Kind::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeCreateEntry {
    pub hostname: String,
    pub description: Option<String>,
    pub label: Option<String>,
    pub room_name: Option<Uuid>,
    pub mount_point: Option<String>,
    pub network_id: Option<uuid::Uuid>,
    pub username: Option<String>,
    pub pasword: Option<String>,
}

impl From<NodeCreateEntry> for Node {
    fn from(value: NodeCreateEntry) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            hostname: value.hostname,
            description: value.description,
            room_name: value.room_name,
            label: value.label,
            mount_point: value.mount_point,
            username: value.username,
            password: value.pasword,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddrCrateEntry {
    pub ip: IpNet,
    pub network_id: Uuid,
    pub status: Option<StatusAddr>,
    pub node_id: Option<Uuid>,
}

impl From<AddrCrateEntry> for Addresses {
    fn from(value: AddrCrateEntry) -> Self {
        Self {
            ip: value.ip,
            network_id: value.network_id,
            status: value.status.unwrap_or_default(),
            node_id: value.node_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSubnet {
    pub prefix: u8,
    pub status: Option<StatusNetwork>,
    pub kind: Option<Kind>,
    pub description: Option<String>,
}

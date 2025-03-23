use super::super::models::{network::Network, node::Node, user::User};
use crate::models::{
    network::{StatusNetwork, Target},
    node::StatusNode,
};
use ipnet::IpNet;
use libipam::types::vlan::VlanId;
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
    pub target: Option<Target>,
}

impl From<NetworkCreateEntry> for Network {
    fn from(value: NetworkCreateEntry) -> Self {
        let avl = value.network.into();
        Self {
            id: Uuid::new_v4(),
            network: value.network,
            description: value.description,
            available: avl,
            used: 0.try_into().unwrap(),
            free: avl,
            vlan: value.vlan,
            father: None,
            children: 0,
            target: value.target.unwrap_or_default(),
            status: StatusNetwork::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NodeCreateEntry {
    pub ip: IpAddr,
    pub description: Option<String>,
    pub label: Option<String>,
    pub room_name: Option<Uuid>,
    pub mount_point: Option<String>,
    pub network_id: uuid::Uuid,
    pub username: Option<String>,
    pub pasword: Option<String>,
}

impl From<NodeCreateEntry> for Node {
    fn from(value: NodeCreateEntry) -> Self {
        Self {
            ip: value.ip,
            description: value.description,
            room_name: value.room_name,
            label: value.label,
            mount_point: value.mount_point,
            status: StatusNode::default(),
            network_id: value.network_id,
            username: value.username,
            password: value.pasword,
        }
    }
}

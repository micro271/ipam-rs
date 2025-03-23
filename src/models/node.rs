use super::{Deserialize, Serialize, Table, Updatable, Uuid};
use ipnet::IpNet;
use macros::FromPgRow;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Deserialize, Serialize, Debug, Updatable, Default)]
pub struct UpdateNode {
    pub ip: Option<IpAddr>,
    pub description: Option<String>,
    pub status: Option<StatusNode>,
    pub network_id: Option<Uuid>,
    pub label: Option<String>,
    pub room: Option<Uuid>,
    pub mount_point: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromPgRow, Table)]
#[table_name("nodes")]
pub struct Node {
    #[FromStr]
    pub ip: IpAddr,

    pub network_id: uuid::Uuid,
    pub description: Option<String>,
    pub label: Option<String>,
    pub room_name: Option<Uuid>,
    pub mount_point: Option<String>,
    pub status: StatusNode,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl std::cmp::PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.network_id == other.network_id
    }
}

impl std::cmp::PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ip.partial_cmp(&other.ip)
    }
}

impl std::cmp::PartialEq<IpAddr> for Node {
    fn eq(&self, other: &IpAddr) -> bool {
        self.ip.eq(other)
    }
}

impl std::cmp::PartialOrd<IpAddr> for Node {
    fn partial_cmp(&self, other: &IpAddr) -> Option<std::cmp::Ordering> {
        self.ip.partial_cmp(other)
    }
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type, PartialEq, Clone, Copy, Default)]
pub enum StatusNode {
    Reserved,

    #[default]
    Unknown,

    Online,
    Offline,
    Reachable,
}

impl From<(IpAddr, uuid::Uuid)> for Node {
    fn from(value: (IpAddr, uuid::Uuid)) -> Self {
        Node {
            ip: value.0,
            description: None,
            label: None,
            room_name: None,
            mount_point: None,
            status: StatusNode::default(),
            network_id: value.1,
            username: None,
            password: None,
        }
    }
}

pub struct NodeRange {
    start: u32,
    end: u32,
    step: u32,
    pub network_id: Uuid,
}

impl NodeRange {
    pub fn new_with_uuid(network: IpNet, network_id: Uuid) -> Result<Self, NodeRangeError> {
        let start = match network.network() {
            IpAddr::V4(e) => u32::from(e),
            IpAddr::V6(_) => return Err(NodeRangeError::InvalidNetwork),
        };

        let host = 2u32.pow(u32::from(network.max_prefix_len() - network.prefix_len())) - 2;

        Ok(NodeRange {
            start,
            end: start + host,
            network_id,
            step: 0,
        })
    }
}

impl Iterator for NodeRange {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        (self.end > self.start + self.step).then(|| {
            self.step += 1;
            Node {
                ip: IpAddr::from(Ipv4Addr::from(self.start + self.step)),
                description: None,
                label: None,
                room_name: None,
                mount_point: None,
                status: StatusNode::default(),
                network_id: self.network_id,
                username: None,
                password: None,
            }
        })
    }
}

impl ExactSizeIterator for NodeRange {
    fn len(&self) -> usize {
        (self.start - self.end) as usize
    }
}

#[derive(Debug)]
pub enum NodeRangeError {
    InvalidNetwork,
}

impl std::fmt::Display for NodeRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeRangeError::InvalidNetwork => write!(f, "Only support ipv4 network"),
        }
    }
}

impl std::error::Error for NodeRangeError {}

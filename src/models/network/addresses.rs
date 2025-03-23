use super::{Deserialize, FromPgRow, Serialize, Table};
use ipnet::IpNet;
use std::net::{IpAddr, Ipv4Addr};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, Table, FromPgRow)]
pub struct Addresses {
    #[FromStr]
    pub ip: IpNet,

    pub network_id: Uuid,
    pub status: StatusAddr,
    pub node_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type, PartialEq, Clone, Copy, Default)]
pub enum StatusAddr {
    Reserved,

    #[default]
    Unknown,

    Online,
    Offline,
    Reachable,
}

pub struct AddrRange {
    start: u32,
    end: u32,
    step: u32,
    pub network_id: Uuid,
    prefix: u8,
}

impl AddrRange {
    pub fn new_with_uuid(network: IpNet, network_id: Uuid) -> Result<Self, AddrRangeError> {
        let start = match network.network() {
            IpAddr::V4(e) => u32::from(e),
            IpAddr::V6(_) => return Err(AddrRangeError::InvalidNetwork),
        };

        let host = 2u32.pow(u32::from(network.max_prefix_len() - network.prefix_len())) - 2;

        Ok(AddrRange {
            start,
            end: start + host,
            network_id,
            prefix: network.prefix_len(),
            step: 0,
        })
    }
}

impl Iterator for AddrRange {
    type Item = Addresses;

    fn next(&mut self) -> Option<Self::Item> {
        (self.end > self.start + self.step).then(|| {
            self.step += 1;
            Addresses {
                ip: IpNet::new(
                    IpAddr::from(Ipv4Addr::from(self.start + self.step)),
                    self.prefix,
                )
                .unwrap(),
                status: StatusAddr::default(),
                network_id: self.network_id,
                node_id: None,
            }
        })
    }
}

impl ExactSizeIterator for AddrRange {
    fn len(&self) -> usize {
        (self.start - self.end) as usize
    }
}

#[derive(Debug)]
pub enum AddrRangeError {
    InvalidNetwork,
}

impl std::fmt::Display for AddrRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddrRangeError::InvalidNetwork => write!(f, "Only support ipv4 network"),
        }
    }
}

impl std::error::Error for AddrRangeError {}

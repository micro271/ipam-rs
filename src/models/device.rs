use super::{Deserialize, Serialize, Table, Updatable, Uuid};
use ipnet::IpNet;
use macros::FromPgRow;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Deserialize, Serialize, Debug, Updatable, Default)]
pub struct UpdateDevice {
    pub ip: Option<IpAddr>,
    pub description: Option<String>,
    pub status: Option<Status>,
    pub network_id: Option<Uuid>,
    pub label: Option<String>,
    pub room: Option<Uuid>,
    pub mount_point: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromPgRow, Table)]
#[table_name("devices")]
pub struct Device {
    #[FromStr]
    pub ip: IpAddr,

    pub description: Option<String>,
    pub label: Option<String>,
    pub room: Option<Uuid>,
    pub mount_point: Option<String>,
    pub status: Status,
    pub network_id: uuid::Uuid,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl std::cmp::PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.network_id == other.network_id
    }
}

impl std::cmp::PartialOrd for Device {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ip.partial_cmp(&other.ip)
    }
}

impl std::cmp::PartialEq<IpAddr> for Device {
    fn eq(&self, other: &IpAddr) -> bool {
        self.ip.eq(other)
    }
}

impl std::cmp::PartialOrd<IpAddr> for Device {
    fn partial_cmp(&self, other: &IpAddr) -> Option<std::cmp::Ordering> {
        self.ip.partial_cmp(other)
    }
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type, PartialEq, Clone, Copy, Default)]
pub enum Status {
    Reserved,

    #[default]
    Unknown,

    Online,
    Offline,
}

impl From<(IpAddr, uuid::Uuid)> for Device {
    fn from(value: (IpAddr, uuid::Uuid)) -> Self {
        Device {
            ip: value.0,
            description: None,
            label: None,
            room: None,
            mount_point: None,
            status: Status::default(),
            network_id: value.1,
            username: None,
            password: None,
        }
    }
}

pub struct DeviceRange {
    start: u32,
    end: u32,
    step: u32,
    network_id: Uuid,
}

impl DeviceRange {
    pub fn set_network_id(&mut self, network_id: Uuid) {
        self.network_id = network_id;
    }
}

impl Iterator for DeviceRange {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end == self.start + self.step {
            None
        } else {
            self.step += 1;
            Some(Device {
                ip: IpAddr::from(Ipv4Addr::from(self.start + self.step)),
                description: None,
                label: None,
                room: None,
                mount_point: None,
                status: Status::default(),
                network_id: self.network_id,
                username: None,
                password: None,
            })
        }
    }
}

impl ExactSizeIterator for DeviceRange {
    fn len(&self) -> usize {
        (self.start - self.end) as usize
    }
}

impl TryFrom<IpNet> for DeviceRange {
    type Error = DeviceRangeError;
    fn try_from(value: IpNet) -> Result<Self, Self::Error> {
        let start = match value.network() {
            IpAddr::V4(e) => u32::from(e),
            IpAddr::V6(_) => return Err(DeviceRangeError::InvalidNetwork),
        };

        let len = 2u32.pow(u32::from(value.max_prefix_len() - value.prefix_len())) - 2;

        Ok(DeviceRange {
            start,
            end: start + len,
            network_id: Uuid::default(),
            step: 0,
        })
    }
}

#[derive(Debug)]
pub enum DeviceRangeError {
    InvalidNetwork,
}

impl std::fmt::Display for DeviceRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceRangeError::InvalidNetwork => write!(f, "Only support ipv4 network"),
        }
    }
}

impl std::error::Error for DeviceRangeError {}

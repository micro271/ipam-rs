use super::*;
use std::net::IpAddr;

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateDevice {
    pub ip: Option<IpAddr>,
    pub description: Option<String>,
    pub office_id: Option<Uuid>,
    pub rack: Option<String>,
    pub room: Option<String>,
    pub status: Option<Status>,
    pub network_id: Option<Uuid>,
    pub credential: Option<Credential>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Device {
    pub ip: IpAddr,
    pub description: Option<String>,
    pub office_id: Option<Uuid>,
    pub rack: Option<String>,
    pub room: Option<String>,
    pub status: Status,
    pub network_id: uuid::Uuid,
    pub credential: Option<Credential>,
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

#[derive(Deserialize, Serialize, Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "CREDENTIAL")]
pub struct Credential {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type, PartialEq, Clone)]
pub enum Status {
    Reserved,
    Unknown,
    Online,
    Offline,
}

impl Default for Status {
    fn default() -> Self {
        Self::Unknown
    }
}

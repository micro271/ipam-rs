use super::*;
use std::net::IpAddr;

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateDevice {
    pub ip: Option<IpAddr>,
    pub description: Option<String>,
    pub office_id: Option<Uuid>,
    pub rack: Option<Option<String>>,
    pub room: Option<Option<String>>,
    pub status: Option<Status>,
    pub network_id: Option<Uuid>,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type, PartialEq)]
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

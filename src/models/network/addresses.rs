use super::{Deserialize, FromPgRow, Serialize, Table};
use ipnet::IpNet;

#[derive(Debug, Deserialize, Serialize, Clone, Table, FromPgRow)]
struct Addresses {
    #[FromStr]
    ip: IpNet,

    network_id: uuid::Uuid,
    status: StatusAddr,
    nose_id: Option<uuid::Uuid>,
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

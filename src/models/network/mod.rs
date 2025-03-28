pub mod addresses;

use super::{Deserialize, FromPgRow, Serialize, Table, Updatable, Uuid};
use addresses::{AddrRange, AddrRangeError};
use ipnet::IpNet;
use libipam::{
    services::ipam::{SubnetList, SubnettingError},
    types::{host_count::HostCount, vlan::VlanId},
};
use macros::MapQuery;

#[derive(Debug, MapQuery, Default)]
pub struct NetworkFilter {
    pub id: Option<Uuid>,
    pub description: Option<String>,
    pub status: Option<StatusNetwork>,
    pub father: Option<Uuid>,
    pub kind: Option<Kind>,
}

#[derive(Debug, Deserialize, Serialize, Updatable)]
pub struct UpdateNetwork {
    pub network: Option<IpNet>,
    pub description: Option<String>,
    pub vlan: Option<VlanId>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Table, FromPgRow)]
#[table_name("networks")]
pub struct Network {
    pub id: Uuid,

    #[FromStr]
    pub subnet: IpNet,

    pub used: HostCount,
    pub free: HostCount,
    pub vlan: Option<VlanId>,
    pub description: Option<String>,
    pub father: Option<Uuid>,
    pub children: i32,
    pub status: StatusNetwork,
    pub kind: Kind,
}

#[derive(Debug, Clone, Copy, sqlx::Type, Deserialize, Serialize, PartialEq, Default)]
#[sqlx(type_name = "KIND_NETWORK")]
pub enum Kind {
    Pool,

    #[default]
    Network,
}

#[derive(Debug, Clone, Copy, sqlx::Type, Deserialize, Serialize, PartialEq, Default)]
#[sqlx(type_name = "STATUS_NETWORK")]
pub enum StatusNetwork {
    #[default]
    Available,

    Reserved,
    Assigned,
    Used,
}

impl std::cmp::PartialEq for Network {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::PartialOrd for Network {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.subnet.partial_cmp(&other.subnet)
    }
}

impl std::cmp::PartialEq<IpNet> for Network {
    fn eq(&self, other: &IpNet) -> bool {
        self.subnet.eq(other)
    }
}

impl std::cmp::PartialOrd<IpNet> for Network {
    fn partial_cmp(&self, other: &IpNet) -> Option<std::cmp::Ordering> {
        self.subnet.partial_cmp(other)
    }
}

impl From<IpNet> for Network {
    fn from(value: IpNet) -> Self {
        let avl = value.into();

        Self {
            subnet: value,
            id: uuid::Uuid::new_v4(),
            used: 0.try_into().unwrap(),
            free: avl,
            vlan: None,
            description: None,
            father: None,
            children: 0,
            status: StatusNetwork::default(),
            kind: Kind::default(),
        }
    }
}

impl Network {
    pub fn addresses(&self) -> Result<AddrRange, AddrRangeError> {
        AddrRange::new_with_uuid(self.subnet, self.id)
    }

    pub fn subnets(&self, prefix: u8) -> Result<SubnetList, SubnettingError> {
        SubnetList::new(self.subnet, prefix)
    }
}

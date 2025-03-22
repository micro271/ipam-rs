use super::{
    Deserialize, FromPgRow, Serialize, Table, Updatable, Uuid,
    node::{DeviceRange, DeviceRangeError},
};
use ipnet::IpNet;
use libipam::{
    services::ipam::{SubnetList, SubnettingError},
    types::{host_count::HostCount, vlan::VlanId},
};

#[derive(Debug, Deserialize, Serialize, Updatable)]
pub struct UpdateNetwork {
    pub network: Option<IpNet>,
    pub description: Option<String>,
    pub vlan: Option<VlanId>,
    pub target: Option<Target>,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Deserialize, Serialize, Clone, Table, FromPgRow)]
#[table_name("networks")]
pub struct Network {
    pub id: Uuid,

    #[FromStr]
    pub network: IpNet,

    pub available: HostCount,
    pub used: HostCount,
    pub free: HostCount,
    pub vlan: Option<VlanId>,
    pub description: Option<String>,
    pub father: Option<Uuid>,
    pub children: i32,
    pub target: Target,
}

#[derive(Debug, Clone, Copy, sqlx::Type, Deserialize, Serialize, PartialEq, Default)]
#[sqlx(type_name = "NETWORKTO")]
pub enum Target {
    Nat,
    #[default]
    Device,
}

impl std::cmp::PartialEq for Network {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::PartialOrd for Network {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.network.partial_cmp(&other.network)
    }
}

impl std::cmp::PartialEq<IpNet> for Network {
    fn eq(&self, other: &IpNet) -> bool {
        self.network.eq(other)
    }
}

impl std::cmp::PartialOrd<IpNet> for Network {
    fn partial_cmp(&self, other: &IpNet) -> Option<std::cmp::Ordering> {
        self.network.partial_cmp(other)
    }
}

impl From<IpNet> for Network {
    fn from(value: IpNet) -> Self {
        let avl = value.into();

        Self {
            network: value,
            id: uuid::Uuid::new_v4(),
            available: avl,
            used: 0.try_into().unwrap(),
            free: avl,
            vlan: None,
            description: None,
            father: None,
            children: 0,
            target: Target::default(),
        }
    }
}

impl Network {
    pub fn devices(&self) -> Result<DeviceRange, DeviceRangeError> {
        DeviceRange::new_with_uuid(self.network, self.id)
    }

    pub fn subnets(&self, prefix: u8) -> Result<SubnetList, SubnettingError> {
        SubnetList::new(self.network, prefix)
    }
}

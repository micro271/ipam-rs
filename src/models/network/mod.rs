pub mod addresses;

use super::{Deserialize, FromPgRow, Serialize, Table, Updatable, Uuid};
use addresses::{AddrRange, AddrRangeError};
use ipnet::IpNet;
use libipam::{
    services::ipam::{SubnetList, SubnettingError},
    types::{host_count::HostCount, vlan::VlanId},
};
use macros::MapQuery;

#[derive(Debug, MapQuery, Default, Clone)]
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

#[derive(Debug, Clone, Copy, Updatable)]
pub struct UpdateHostCount {
    #[IgnoreFieldToUpdate]
    subnet: IpNet,

    used: HostCount,
    free: HostCount,
}

impl UpdateHostCount {
    fn new(subnet: IpNet, used: HostCount, free: HostCount) -> Self {
        Self { subnet, used, free }
    }

    pub fn less_free_more_used(&mut self, n: u32) {
        self.used = if self.used.is_max() {
            HostCount::new_from_ipnet_with_add(self.subnet, n).unwrap_or(HostCount::new_max())
        } else {
            self.used.add(n)
        };

        self.free = if self.free.is_max() {
            HostCount::new_from_ipnet_with_sub(self.subnet, n)
                .unwrap_or(HostCount::try_from(0).unwrap())
        } else {
            self.free.sub(n)
        };
    }

    pub fn less_used_more_free(&mut self, n: u32) {
        self.used = if self.used.is_max() {
            HostCount::new_from_ipnet_with_sub(self.subnet, n).unwrap_or(HostCount::new_max())
        } else {
            self.used.sub(n)
        };

        self.free = if self.used.is_max() {
            HostCount::new_from_ipnet_with_add(self.subnet, n)
                .unwrap_or(HostCount::try_from(0).unwrap())
        } else {
            self.used.add(n)
        }
    }
}

pub struct NetworkSubnetList {
    iter: SubnetList,
    default: DefaultValuesNetwork,
}

impl NetworkSubnetList {
    pub fn new(iter_subnet: SubnetList, default_values: Option<DefaultValuesNetwork>) -> Self {
        Self {
            iter: iter_subnet,
            default: default_values.unwrap_or_default(),
        }
    }

    pub fn set_default_values(&mut self, values: DefaultValuesNetwork) {
        self.default = values;
    }

    pub fn batch(self, window: usize) -> NetworkSubnetBatch {
        NetworkSubnetBatch::new(self, window)
    }
}

impl std::iter::Iterator for NetworkSubnetList {
    type Item = Network;

    fn next(&mut self) -> Option<Self::Item> {
        let net = self.iter.next()?;
        let avl = HostCount::from(net);

        Some(Network {
            id: Uuid::new_v4(),
            subnet: net,
            used: 0.try_into().unwrap(),
            free: avl,
            vlan: None,
            description: self.default.description.clone(),
            father: self.default.father,
            children: 0,
            status: self.default.status.unwrap_or_default(),
            kind: self.default.kind.unwrap_or_default(),
        })
    }
}

impl std::iter::ExactSizeIterator for NetworkSubnetList {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

#[derive(Default)]
pub struct DefaultValuesNetwork {
    pub father: Option<Uuid>,
    pub status: Option<StatusNetwork>,
    pub kind: Option<Kind>,
    pub description: Option<String>,
}

pub struct NetworkSubnetBatch {
    iter: NetworkSubnetList,
    window: usize,
}

impl NetworkSubnetBatch {
    pub fn new(iter: NetworkSubnetList, window: usize) -> Self {
        Self { iter, window }
    }
}

impl std::iter::Iterator for NetworkSubnetBatch {
    type Item = Vec<Network>;

    fn next(&mut self) -> Option<Self::Item> {
        let tmp = self.iter.by_ref().take(self.window).collect::<Vec<_>>();

        (!tmp.is_empty()).then_some(tmp)
    }
}

impl std::iter::ExactSizeIterator for NetworkSubnetBatch {
    fn len(&self) -> usize {
        self.iter.len().div_ceil(self.window)
    }
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

    pub fn subnets(&self, prefix: u8) -> Result<NetworkSubnetList, SubnettingError> {
        Ok(NetworkSubnetList::new(
            SubnetList::new(self.subnet, prefix)?,
            None,
        ))
    }
    pub fn update_host_count(&self) -> UpdateHostCount {
        UpdateHostCount::new(self.subnet, self.used, self.free)
    }
}

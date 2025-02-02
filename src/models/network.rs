use super::{
    device::{DeviceRange, DeviceRangeError},
    *,
};
use ipnet::IpNet;
use libipam::type_net::{host_count::HostCount, vlan::VlanId};

#[derive(Debug, Deserialize, Serialize, Updatable)]
pub struct UpdateNetwork {
    pub network: Option<IpNet>,
    pub description: Option<String>,
    pub vlan: Option<VlanId>,
    pub to: Option<To>,
}

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
    pub to: To,
}

#[derive(Debug, Clone, Copy, sqlx::Type, Deserialize, Serialize, PartialEq, Default)]
pub enum To {
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
        let avl = HostCount::new((&value).into());

        Self {
            network: value,
            id: uuid::Uuid::new_v4(),
            available: avl.clone(),
            used: 0.into(),
            free: avl,
            vlan: None,
            description: None,
            father: None,
            children: 0,
            to: To::default(),
        }
    }
}

impl Network {
    pub fn devices(&self) -> Result<DeviceRange, DeviceRangeError> {
        let mut aux: DeviceRange = self.network.try_into()?;
        aux.set_network_id(self.id);
        Ok(aux)
    }
}

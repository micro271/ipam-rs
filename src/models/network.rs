use super::*;
use ipnet::IpNet;
use libipam::type_net::{host_count::HostCount, vlan::Vlan};

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateNetwork {
    pub network: Option<IpNet>,
    pub description: Option<String>,
    pub vlan: Option<Vlan>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Network {
    pub id: Uuid,
    pub vlan: Option<Vlan>,
    pub network: IpNet,
    pub description: Option<String>,
    pub available: HostCount,
    pub used: HostCount,
    pub free: HostCount,
}

impl std::cmp::PartialEq for Network {
    fn eq(&self, other: &Self) -> bool {
        self.network == other.network && self.vlan == other.vlan /* Location of network? */
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

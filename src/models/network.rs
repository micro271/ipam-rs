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

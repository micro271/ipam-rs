use libipam::type_net::vlan::VlanId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Vlan {
    pub id: Option<VlanId>,
    pub description: Option<String>,
}

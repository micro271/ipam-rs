use libipam::type_net::vlan::VlanId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VlanRow {
    pub id: VlanId,
    pub description: String,
}

pub struct Vlan {
    pub id: Option<VlanId>,
    pub description: Option<String>,
}

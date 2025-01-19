use libipam::type_net::vlan::VlanId;
use serde::{Deserialize, Serialize};
use crate::database::repository::{Table, TypeTable};
use macros::Table as Tb;

#[derive(Deserialize, Serialize, Debug, Clone, Tb)]
#[table_name = "vlans"]
pub struct Vlan {
    pub id: VlanId,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateVlan {
    pub id: Option<VlanId>,
    pub description: Option<String>,
}

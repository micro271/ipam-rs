use super::*;
use libipam::type_net::vlan::VlanId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Table, FromPgRow)]
#[table_name("vlans")]
pub struct Vlan {
    pub id: VlanId,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Updatable)]
pub struct UpdateVlan {
    pub id: Option<VlanId>,
    pub description: Option<String>,
}

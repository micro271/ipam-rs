use super::{FromPgRow, Table, Updatable};
use libipam::types::vlan::VlanId;
use macros::MapQuery;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Table, FromPgRow)]
#[table_name("vlans")]
pub struct Vlan {
    pub id: VlanId,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Default, Updatable)]
pub struct UpdateVlan {
    pub id: Option<VlanId>,
    pub description: Option<String>,
}

#[derive(Debug, Default, MapQuery)]
pub struct VlanCondition {
    pub id: Option<VlanId>,
    pub description: Option<String>,
}

impl VlanCondition {
    pub fn p_key(id: VlanId) -> Self {
        Self {
            id: Some(id),
            description: None,
        }
    }
}

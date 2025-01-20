use crate::database::repository::{Table, TypeTable, Updatable};
use libipam::type_net::vlan::VlanId;
use macros::{Table as Tb, Updatable as Upd};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone, Tb)]
#[table_name = "vlans"]
pub struct Vlan {
    pub id: VlanId,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Upd)]
pub struct UpdateVlan {
    pub id: Option<VlanId>,
    pub description: Option<String>,
}

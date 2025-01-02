use super::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Location {
    pub label: String,
    pub mont_point: String,
    pub id_room: String,
    pub address: String,
}

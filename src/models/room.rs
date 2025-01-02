use super::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Room {
    pub id: String,
    pub address: String,
}

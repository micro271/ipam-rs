use super::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Room {
    pub id: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoom {
    pub id: Option<String>,
    pub address: Option<String>,
}

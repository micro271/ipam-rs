use super::*;

#[derive(Debug, Deserialize, Serialize, Clone, Table, FromPgRow)]
pub struct Room {
    pub id: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Updatable)]
pub struct UpdateRoom {
    pub id: Option<String>,
    pub address: Option<String>,
}

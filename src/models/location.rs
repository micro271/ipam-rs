use super::{Deserialize, FromPgRow, Serialize, Table, Updatable};

#[derive(Debug, Serialize, Deserialize, Clone, Table, FromPgRow)]
#[table_name("locations")]
pub struct Location {
    pub label: String,
    pub mont_point: String,
    pub room_name: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Updatable)]
pub struct LocationUpdate {
    pub label: Option<String>,
    pub mont_point: Option<String>,
    pub room_name: Option<String>,
    pub address: Option<String>,
}

use super::{Deserialize, Serialize, Table, Updatable, Uuid};
use macros::{FromPgRow, MapQuery};

#[derive(Deserialize, Serialize, Debug, Updatable, Default)]
pub struct UpdateNode {
    pub hostname: Option<String>,
    pub description: Option<String>,
    pub label: Option<String>,
    pub room: Option<Uuid>,
    pub mount_point: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, FromPgRow, Table)]
#[table_name("nodes")]
pub struct Node {
    pub id: Uuid,
    pub hostname: String,
    pub description: Option<String>,
    pub label: Option<String>,
    pub room_name: Option<Uuid>,
    pub mount_point: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, Clone, MapQuery, Default)]
pub struct NodeFilter {
    pub id: Option<Uuid>,
    pub hostname: Option<String>,
    pub description: Option<String>,
    pub network_id: Option<Uuid>,
    pub label: Option<String>,
    pub room_name: Option<Uuid>,
    pub mount_point: Option<String>,
}

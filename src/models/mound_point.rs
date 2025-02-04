use super::{Deserialize, FromPgRow, Serialize, Table, Updatable};

#[derive(Debug, Clone, Deserialize, Serialize, Table, FromPgRow)]
#[table_name("mount_point")]
pub struct MountPoint {
    pub name: String,
}

#[derive(Debug, Deserialize, Updatable)]
pub struct UpdateMountPoint {
    pub name: Option<String>,
}

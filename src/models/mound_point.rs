use super::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MountPoint {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMountPoint {
    pub name: Option<String>,
}

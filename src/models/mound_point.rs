use super::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct MountPoint {
    pub name: String,
}

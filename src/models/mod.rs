pub mod device;
pub mod location;
pub mod mound_point;
pub mod network;
pub mod room;
pub mod user;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod office {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Office {
        pub id: Uuid,
        pub name: String,
        pub address: Option<String>,
        pub description: Option<String>,
    }

    pub struct UpdateOffice {
        pub description: Option<String>,
        pub address: Option<String>,
    }
}

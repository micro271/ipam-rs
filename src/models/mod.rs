pub mod device;
pub mod location;
pub mod mound_point;
pub mod network;
pub mod office;
pub mod room;
pub mod user;
pub mod vlan;

use serde::{Deserialize, Serialize};
use macros::{FromPgRow, Table, Updatable};
use uuid::Uuid;

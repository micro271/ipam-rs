pub mod network;
pub mod node;
pub mod user;
pub mod vlan;

use macros::{FromPgRow, Table, Updatable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

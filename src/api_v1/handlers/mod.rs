pub mod addresses;
pub mod auth;
mod entries;
pub mod error;
pub mod extractors;
pub mod network;
pub mod node;
pub mod vlan;

use crate::{
    app_state::StateType,
    database::repository::{QueryResult, Repository},
    models::{self, user::Role},
    response::ResponseQuery,
};
use axum::{
    extract::{Json, Path, Query, State},
    http::{StatusCode, Uri},
};
use entries::params::PaginationParams;
use extractors::IsAdministrator;
use libipam::response_error::ResponseError;
use tracing::{Level, instrument};
use uuid::Uuid;

type ResponseDefault<T> = Result<ResponseQuery<T, serde_json::Value>, ResponseError>;

const BATCH_SIZE: usize = 8192;

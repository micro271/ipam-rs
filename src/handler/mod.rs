pub mod addresses;
pub mod auth;
mod entries;
pub mod error;
pub mod extractors;
pub mod network;
pub mod node;
pub mod vlan;

use crate::{
    database::{
        RepositoryInjection,
        repository::{QueryResult, Repository},
    },
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
use std::sync::Arc;
use tracing::{Level, instrument};
use uuid::Uuid;

type RepositoryType = Arc<RepositoryInjection<sqlx::postgres::Postgres>>;

type ResponseDefault<T> = Result<ResponseQuery<T, serde_json::Value>, ResponseError>;

const BATCH_SIZE: usize = 8192;

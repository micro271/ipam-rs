pub mod addresses;
pub mod auth;
mod entries;
pub mod error;
pub mod extractors;
pub mod location;
pub mod mount_point;
pub mod network;
pub mod node;
pub mod office;
pub mod room;
pub mod vlan;

use crate::{
    database::{
        RepositoryInjection,
        repository::{QueryResult, Repository},
    },
    models::{self, user::Role},
};
use axum::{
    extract::{Json, Path, Query, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use entries::params::PaginationParams;
use extractors::IsAdministrator;
use libipam::response_error::ResponseError;
use std::{collections::HashMap, sync::Arc};
use tracing::{Level, instrument};
use uuid::Uuid;

type RepositoryType = Arc<RepositoryInjection<sqlx::postgres::Postgres>>;

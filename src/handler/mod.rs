pub mod auth;
pub mod device;
mod entries;
pub mod error;
pub mod extractors;
pub mod location;
pub mod mount_point;
pub mod network;
pub mod room;
pub mod vlan;

use crate::{
    database::{repository::Repository, RepositoryInjection},
    models::{self, user::Role},
};

use axum::{
    extract::{Json, Path, Query, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use extractors::IsAdministrator;
use libipam::response_error::ResponseError;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

type RepositoryType = Arc<RepositoryInjection<sqlx::postgres::Postgres>>;

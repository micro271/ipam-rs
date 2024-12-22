pub mod auth;
pub mod device;
pub mod error;
pub mod extractors;
mod models_data_entry;
pub mod network;
mod params;

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
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

type RepositoryType = Arc<RepositoryInjection<sqlx::postgres::Postgres>>;

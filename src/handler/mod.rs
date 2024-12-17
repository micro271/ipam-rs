pub mod auth;
pub mod device;
pub mod error;
mod models_data_entry;
pub mod network;
pub mod extractors;

use crate::{
    database::{
        repository::Repository,
        PgRepository,
    },
    models::{self, user::Role},
};

use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use libipam::response_error::ResponseError;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;
use extractors::IsAdministrator;

type RepositoryType = Arc<Mutex<PgRepository>>;

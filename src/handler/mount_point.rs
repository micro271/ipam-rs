use std::collections::HashMap;

use super::{entries::params::PaginationParams, RepositoryType, ResponseError};
use crate::{
    database::repository::{QueryResult, Repository},
    models::mound_point::{MountPoint, UpdateMountPoint},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};

pub async fn get(
    State(state): State<RepositoryType>,
    Path(name): Path<Option<String>>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>
) -> Result<QueryResult<MountPoint>, ResponseError> {
    Ok(state
        .get::<MountPoint>(name.map(|x| HashMap::from([("name", x.into())])), limit, offset)
        .await?
        .into())
}

pub async fn update(
    State(state): State<RepositoryType>,
    Path(name): Path<String>,
    Json(updater): Json<UpdateMountPoint>,
) -> Result<QueryResult<MountPoint>, ResponseError> {
    Ok(state
        .update::<MountPoint, _>(updater, Some(HashMap::from([("name", name.into())])))
        .await?)
}

pub async fn delete(
    State(state): State<RepositoryType>,
    Path(name): Path<String>,
) -> Result<QueryResult<MountPoint>, ResponseError> {
    Ok(state
        .delete::<MountPoint>(Some(HashMap::from([("name", name.into())])))
        .await?)
}

pub async fn insert(
    State(state): State<RepositoryType>,
    Json(new): Json<MountPoint>,
) -> Result<QueryResult<MountPoint>, ResponseError> {
    Ok(state.insert(new).await?)
}

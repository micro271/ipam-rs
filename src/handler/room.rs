use axum::extract::Query;

use super::{
    Json, Level, RepositoryType, ResponseError, State,
    entries::params::{PaginationParams, ParamRoom, ParamRoomStrict},
    instrument,
};
use crate::{
    database::repository::{QueryResult, Repository},
    models::room::{Room, UpdateRoom},
};

#[instrument(level = Level::DEBUG)]
pub async fn insert(
    State(state): State<RepositoryType>,
    Json(room): Json<Room>,
) -> Result<QueryResult<Room>, ResponseError> {
    Ok(state.insert(room).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    Query(param): Query<ParamRoomStrict>,
    Json(room): Json<UpdateRoom>,
) -> Result<QueryResult<Room>, ResponseError> {
    Ok(state.update::<Room, _>(room, param).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(param): Query<ParamRoom>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Room>, ResponseError> {
    Ok(state.get(param, limit, offset).await?)
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Query(param): Query<ParamRoomStrict>,
) -> Result<QueryResult<Room>, ResponseError> {
    Ok(state.delete::<Room>(param).await?)
}

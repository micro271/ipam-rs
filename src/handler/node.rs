use super::{
    IntoResponse, IsAdministrator, Json, Level, PaginationParams, Path, Query, Repository,
    RepositoryType, ResponseError, State, StatusCode, Uuid, entries, instrument,
};
use crate::{
    database::{repository::QueryResult, transaction::Transaction},
    models::{
        network::{Kind, Network},
        node::{Node, UpdateNode},
    },
};
use entries::{
    models::NodeCreateEntry,
    params::{ParamsDevice, ParamsDeviceStrict},
};
use libipam::services::ipam::Ping;

#[instrument(level = Level::DEBUG)]
pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(node): Json<NodeCreateEntry>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.insert::<Node>(node.into()).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
    Json(new): Json<UpdateNode>,
) -> Result<StatusCode, ResponseError> {
    todo!()
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(params): Query<ParamsDevice>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Node>, ResponseError> {
    Ok(state.get::<Node>(params, limit, offset).await?.into())
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.delete::<Node>(param).await?)
}

#[instrument(level = Level::INFO)]
pub async fn ping(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(condition): Query<ParamsDeviceStrict>,
) -> Result<Ping, ResponseError> {
    todo!()
}

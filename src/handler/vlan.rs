use super::{instrument, Level, PaginationParams, RepositoryType, ResponseError, State};
use crate::{
    database::repository::{QueryResult, Repository},
    models::vlan::{UpdateVlan, Vlan},
};
use axum::{
    extract::{Path, Query},
    Json,
};
use libipam::type_net::vlan::VlanId;

#[instrument(level = Level::DEBUG)]
pub async fn insert(
    State(state): State<RepositoryType>,
    Json(vlan): Json<Vlan>,
) -> Result<QueryResult<Vlan>, ResponseError> {
    Ok(state.insert(vlan).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Vlan>, ResponseError> {
    Ok(state
        .get::<Vlan>(Some([("id", id.into())].into()), limit, offset)
        .await?
        .into())
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
    Json(vlan): Json<UpdateVlan>,
) -> Result<QueryResult<Vlan>, ResponseError> {
    Ok(state
        .update::<Vlan, _>(vlan, Some([("id", id.into())].into()))
        .await?)
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
) -> Result<QueryResult<Vlan>, ResponseError> {
    Ok(state.delete(Some([("id", id.into())].into())).await?)
}

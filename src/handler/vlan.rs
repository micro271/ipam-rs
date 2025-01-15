use std::collections::HashMap;

use super::{RepositoryType, ResponseError, State};
use crate::{
    database::repository::{QueryResult, Repository},
    models::vlan::{UpdateVlan, Vlan},
};
use axum::{extract::Path, Json};
use libipam::type_net::vlan::VlanId;

pub async fn insert(
    State(state): State<RepositoryType>,
    Json(vlan): Json<Vlan>,
) -> Result<QueryResult<Vlan>, ResponseError> {
    Ok(state.insert(vlan).await?)
}

pub async fn get(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
) -> Result<QueryResult<Vlan>, ResponseError> {
    Ok(state
        .get::<Vlan>(Some(HashMap::from([("id", id.into())])))
        .await?
        .into())
}

pub async fn update(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
    Json(vlan): Json<UpdateVlan>,
) -> Result<QueryResult<Vlan>, ResponseError> {
    Ok(state
        .update::<Vlan, _>(vlan, Some(HashMap::from([("id", id.into())])))
        .await?)
}

pub async fn delete(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
) -> Result<QueryResult<Vlan>, ResponseError> {
    Ok(state
        .delete(Some(HashMap::from([("id", id.into())])))
        .await?)
}

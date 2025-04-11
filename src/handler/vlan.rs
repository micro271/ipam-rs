use super::{Level, PaginationParams, RepositoryType, ResponseDefault, State, instrument};
use crate::{
    database::repository::Repository,
    models::vlan::{UpdateVlan, Vlan, VlanCondition},
    response::ResponseQuery,
};
use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
};
use libipam::types::vlan::VlanId;
use serde_json::json;

#[instrument(level = Level::DEBUG)]
pub async fn insert(
    State(state): State<RepositoryType>,
    Json(vlan): Json<Vlan>,
) -> ResponseDefault<()> {
    Ok(state.insert(vlan).await?.into())
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> ResponseDefault<Vec<Vlan>> {
    let resp = state
        .get::<Vlan>(VlanCondition::p_key(id), limit, offset)
        .await?;

    let metadata = Some(json!({
        "length": resp.len(),
        "success": true,
        "status": StatusCode::OK.as_u16(),
    }));

    Ok(ResponseQuery::new(
        Some(resp),
        metadata,
        None,
        StatusCode::OK,
    ))
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
    Json(vlan): Json<UpdateVlan>,
) -> ResponseDefault<()> {
    Ok(state
        .update::<Vlan, _>(vlan, VlanCondition::p_key(id))
        .await?
        .into())
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Path(id): Path<VlanId>,
) -> ResponseDefault<()> {
    Ok(state.delete::<Vlan>(VlanCondition::p_key(id)).await?.into())
}

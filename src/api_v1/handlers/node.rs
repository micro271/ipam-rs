use super::{
    IsAdministrator, Json, PaginationParams, Path, Query, Repository, ResponseDefault, State,
    StateType, Uuid, entries,
};
use crate::{
    models::node::{Node, NodeCondition, UpdateNode},
    response::ResponseQuery,
};
use axum::http::StatusCode;
use entries::models::NodeCreateEntry;
use serde_json::json;

pub async fn create(
    State(state): State<StateType>,
    _: IsAdministrator,
    Json(node): Json<NodeCreateEntry>,
) -> ResponseDefault<()> {
    Ok(state.insert::<Node>(node.into()).await?.into())
}

pub async fn update(
    State(state): State<StateType>,
    _: IsAdministrator,
    Path(id): Path<Uuid>,
    Json(new): Json<UpdateNode>,
) -> ResponseDefault<()> {
    Ok(state
        .update::<Node, _>(new, NodeCondition::p_key(id))
        .await?
        .into())
}

pub async fn get(
    State(state): State<StateType>,
    Query(params): Query<NodeCondition>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> ResponseDefault<Vec<Node>> {
    let data = state.get::<Node>(params, limit, offset).await?;

    let metadata = Some(json!({
        "length": data.len(),
        "success": true,
        "status": StatusCode::OK.as_u16(),
    }));

    Ok(ResponseQuery::new(
        Some(data),
        metadata,
        None,
        StatusCode::OK,
    ))
}

pub async fn delete(
    State(state): State<StateType>,
    _: IsAdministrator,
    Path(id): Path<Uuid>,
) -> ResponseDefault<()> {
    Ok(state.delete::<Node>(NodeCondition::p_key(id)).await?.into())
}

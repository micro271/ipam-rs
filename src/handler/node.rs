use super::{
    IntoResponse, IsAdministrator, Json, Level, PaginationParams, Path, Query, Repository,
    RepositoryType, ResponseError, State, Uuid, entries, instrument,
};
use crate::{
    database::repository::QueryResult,
    models::node::{Node, NodeFilter, UpdateNode},
};
use entries::models::NodeCreateEntry;

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
    Path(id): Path<Uuid>,
    Json(new): Json<UpdateNode>,
) -> Result<QueryResult<Node>, ResponseError> {
    Ok(state
        .update::<Node, _>(
            new,
            NodeFilter {
                id: Some(id),
                ..Default::default()
            },
        )
        .await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(params): Query<NodeFilter>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Node>, ResponseError> {
    Ok(state.get::<Node>(params, limit, offset).await?)
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state
        .delete::<Node>(NodeFilter {
            id: Some(id),
            ..Default::default()
        })
        .await?)
}

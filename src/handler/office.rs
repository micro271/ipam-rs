use super::*;
use crate::database::repository::QueryResult;
use entries::params::OfficeParam;
use models::office::{Office, UpdateOffice};

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
    Json(updater): Json<UpdateOffice>,
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state
        .update::<Office, _>(updater, Some([("id", id.into())].into()))
        .await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(of): Query<OfficeParam>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state.get::<Office>(of, limit, offset).await?.into())
}

#[instrument(level = Level::DEBUG)]
pub async fn insert(
    State(state): State<RepositoryType>,
    Json(off): Json<Office>,
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state.insert::<Office>(off).await?)
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state
        .delete::<Office>(Some([("id", id.into())].into()))
        .await?)
}

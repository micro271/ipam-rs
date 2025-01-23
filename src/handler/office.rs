use super::*;
use crate::database::repository::QueryResult;
use entries::params::OfficeParam;
use models::office::{Office, UpdateOffice};

pub async fn update(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
    Json(updater): Json<UpdateOffice>,
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state
        .update::<'_, Office, _>(updater, Some(HashMap::from([("id", id.into())])))
        .await?)
}

pub async fn get(
    State(state): State<RepositoryType>,
    Query(of): Query<OfficeParam>,
    Query(_pagination): Query<PaginationParams>,
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state.get::<Office>(of.get_pairs()).await?.into())
}

pub async fn insert(
    State(state): State<RepositoryType>,
    Json(off): Json<Office>,
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state.insert::<Office>(off).await?.into())
}

pub async fn delete(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state.delete::<Office>(Some(HashMap::from([("id", id.into())]))).await?.into())
}
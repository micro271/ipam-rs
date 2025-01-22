use super::*;
use crate::database::repository::QueryResult;
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
) -> Result<QueryResult<Office>, ResponseError> {
    Ok(state.get::<Office>(None).await?.into())
}
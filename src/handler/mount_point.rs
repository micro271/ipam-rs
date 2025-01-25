use super::*;
use models::mound_point::{MountPoint, UpdateMountPoint};

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Path(name): Path<Option<String>>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<MountPoint>, ResponseError> {
    Ok(state
        .get::<MountPoint>(
            name.map(|x| HashMap::from([("name", x.into())])),
            limit,
            offset,
        )
        .await?
        .into())
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    Path(name): Path<String>,
    Json(updater): Json<UpdateMountPoint>,
) -> Result<QueryResult<MountPoint>, ResponseError> {
    Ok(state
        .update::<MountPoint, _>(updater, Some(HashMap::from([("name", name.into())])))
        .await?)
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Path(name): Path<String>,
) -> Result<QueryResult<MountPoint>, ResponseError> {
    Ok(state
        .delete::<MountPoint>(Some(HashMap::from([("name", name.into())])))
        .await?)
}

#[instrument(level = Level::INFO)]
pub async fn insert(
    State(state): State<RepositoryType>,
    Json(new): Json<MountPoint>,
) -> Result<QueryResult<MountPoint>, ResponseError> {
    Ok(state.insert(new).await?)
}

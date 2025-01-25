use super::*;
use entries::params::{LocationParam, LocationParamStict};
use models::location::{Location, LocationUpdate};

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParam>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state
        .get::<Location>(param.get_pairs(), limit, offset)
        .await?
        .into())
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParamStict>,
    Json(updater): Json<LocationUpdate>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state
        .update::<Location, _>(updater, param.get_pairs())
        .await?)
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParamStict>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.delete::<Location>(param.get_pairs()).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn insert(
    State(state): State<RepositoryType>,
    Json(new): Json<Location>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.insert(new).await?)
}

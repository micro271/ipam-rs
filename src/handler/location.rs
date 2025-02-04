use super::{
    entries, instrument, models, Json, Level, PaginationParams, Query, QueryResult, Repository,
    RepositoryType, ResponseError, State,
};
use entries::params::{LocationParam, LocationParamStict};
use models::location::{Location, LocationUpdate};

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParam>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.get::<Location>(param, limit, offset).await?.into())
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParamStict>,
    Json(updater): Json<LocationUpdate>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.update::<Location, _>(updater, param).await?)
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParamStict>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.delete::<Location>(param).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn insert(
    State(state): State<RepositoryType>,
    Json(new): Json<Location>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.insert(new).await?)
}

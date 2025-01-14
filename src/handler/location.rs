use super::{
    entries::params::{GetMapParams, LocationParam, LocationParamStict},
    RepositoryType, ResponseError,
};
use crate::{
    database::repository::{QueryResult, Repository},
    models::location::{Location, LocationUpdate},
};
use axum::{
    extract::{Query, State},
    Json,
};

pub async fn get(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParam>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.get::<Location>(param.get_pairs()).await?.into())
}

pub async fn update(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParamStict>,
    Json(updater): Json<LocationUpdate>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state
        .update::<Location, _>(updater, param.get_pairs())
        .await?)
}

pub async fn delete(
    State(state): State<RepositoryType>,
    Query(param): Query<LocationParamStict>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.delete::<Location>(param.get_pairs()).await?)
}

pub async fn insert(
    State(state): State<RepositoryType>,
    Json(new): Json<Location>,
) -> Result<QueryResult<Location>, ResponseError> {
    Ok(state.insert(new).await?)
}

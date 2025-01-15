use axum::extract::Query;

use super::{entries::params::{GetMapParams, ParamRoom, ParamRoomStrict}, Json, RepositoryType, ResponseError, State};
use crate::{database::repository::{QueryResult, Repository}, models::room::{Room, UpdateRoom}};



pub async fn insert(State(state): State<RepositoryType>, Json(room): Json<Room>) -> Result<QueryResult<Room>, ResponseError> {
    Ok(state.insert(room).await?)
}

pub async fn update(State(state): State<RepositoryType>, Query(param): Query<ParamRoomStrict>, Json(room): Json<UpdateRoom>) -> Result<QueryResult<Room>, ResponseError> {
    Ok(state.update::<Room, _>(room, param.get_pairs()).await?)
}

pub async fn get(State(state): State<RepositoryType>, Query(param): Query<ParamRoom>) -> Result<QueryResult<Room>, ResponseError> {
    Ok(state.get(param.get_pairs()).await?.into())
}

pub async fn delete(State(state): State<RepositoryType>, Query(param): Query<ParamRoomStrict>) -> Result<QueryResult<Room>, ResponseError> {
    Ok(state.delete::<Room>(param.get_pairs()).await?)
}
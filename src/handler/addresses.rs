use super::{
    Json, Query, RepositoryType, State,
    entries::{
        models::AddrCrateEntry,
        params::{PaginationParams, ParamAddresse, ParamAddresseStrict},
    },
};
use crate::{
    database::repository::{QueryResult, Repository},
    models::network::addresses::{AddrUpdate, Addresses},
};
use axum::http::StatusCode;
use libipam::response_error::ResponseError;

type Resp = Result<QueryResult<Addresses>, ResponseError>;

pub async fn insert(
    State(state): State<RepositoryType>,
    Json(new_addr): Json<AddrCrateEntry>,
) -> Resp {
    Ok(state.insert::<Addresses>(new_addr.into()).await?)
}

pub async fn update(State(state): State<RepositoryType>, Json(_): Json<AddrUpdate>) -> Resp {
    Err(ResponseError::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .build())
}

pub async fn get(
    State(state): State<RepositoryType>,
    Query(PaginationParams { limit, offset }): Query<PaginationParams>,
    Query(param): Query<ParamAddresse>,
) -> Resp {
    let mut addrs = state.get::<Addresses>(param, limit, offset).await?;

    addrs.sort_by_key(|addr| addr.ip);

    Ok(addrs.into())
}

pub async fn delete(
    State(state): State<RepositoryType>,
    Query(param): Query<ParamAddresseStrict>,
) -> Resp {
    Ok(state.delete(param).await?)
}

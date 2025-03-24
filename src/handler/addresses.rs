use super::{
    Json, Query, RepositoryType, State,
    entries::{
        models::AddrCrateEntry,
        params::{PaginationParams, ParamAddrFilter},
    },
    extractors::IsAdministrator,
};
use crate::{
    database::repository::{QueryResult, Repository},
    models::network::addresses::{AddrCondition, AddrUpdate, Addresses},
};
use axum::{extract::Path, http::StatusCode};
use ipnet::IpNet;
use libipam::response_error::ResponseError;
use uuid::Uuid;

type Resp = Result<QueryResult<Addresses>, ResponseError>;

pub async fn insert(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(new_addr): Json<AddrCrateEntry>,
) -> Resp {
    Ok(state.insert::<Addresses>(new_addr.into()).await?)
}

pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(updater): Json<AddrUpdate>,
) -> Resp {
    Err(ResponseError::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .build())
}

pub async fn get(
    State(state): State<RepositoryType>,
    Query(PaginationParams { limit, offset }): Query<PaginationParams>,
    Path(network_id): Path<Uuid>,
    Query(ParamAddrFilter {
        ip,
        node_id,
        status,
    }): Query<ParamAddrFilter>,
) -> Resp {
    let mut addrs = state
        .get::<Addresses>(
            AddrCondition {
                network_id,
                ip,
                node_id,
                status,
            },
            limit,
            offset,
        )
        .await?;

    addrs.sort_by_key(|addr| addr.ip);

    Ok(addrs.into())
}

pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(network_id): Path<Uuid>,
    Query(ip): Query<IpNet>,
) -> Resp {
    Ok(state
        .delete(AddrCondition {
            network_id,
            ip: Some(ip),
            ..Default::default()
        })
        .await?)
}

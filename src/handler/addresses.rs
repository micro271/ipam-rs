use super::{
    Json, Path, Query, RepositoryType, State,
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
    Path(network_id): Path<Uuid>,
    Query(ip): Query<IpNet>,
    Json(updater): Json<AddrUpdate>,
) -> Resp {
    let resp = state
        .update::<Addresses, _>(
            updater,
            AddrCondition {
                ip: Some(ip),
                network_id: Some(network_id),
                ..Default::default()
            },
        )
        .await?;

    Ok(resp)
}

pub async fn get(
    State(state): State<RepositoryType>,
    Query(PaginationParams { limit, offset }): Query<PaginationParams>,
    Path(network_id): Path<Uuid>,
    Query(ParamAddrFilter {
        ip,
        node_id,
        status,
        sort,
    }): Query<ParamAddrFilter>,
) -> Resp {
    let mut addrs = state
        .get::<Addresses>(
            AddrCondition {
                network_id: Some(network_id),
                ip,
                node_id,
                status,
            },
            limit,
            offset,
        )
        .await?;

    if let Some(true) = sort {
        addrs.get_mut_data().unwrap().sort_by_key(|x| x.ip);
    }

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
            network_id: Some(network_id),
            ip: Some(ip),
            ..Default::default()
        })
        .await?)
}

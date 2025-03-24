use super::{
    Json, Path, Query, RepositoryType, State,
    entries::{
        models::AddrCrateEntry,
        params::{PaginationParams, ParamAddrFilter},
    },
    extractors::IsAdministrator,
};
use crate::{
    database::{
        repository::{QueryResult, Repository},
        transaction::Transaction,
    },
    models::network::{
        Kind, Network, NetworkFilter,
        addresses::{AddrCondition, AddrUpdate, Addresses},
    },
};
use axum::http::StatusCode;
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

pub async fn create_all_ip_addresses(
    State(state): State<RepositoryType>,
    Path(network_id): Path<Uuid>,
) -> Resp {
    let network = state
        .get::<Network>(
            NetworkFilter {
                id: Some(network_id),
                ..Default::default()
            },
            None,
            None,
        )
        .await?
        .take_data()
        .unwrap()
        .remove(0);

    if network.kind != Kind::Network {
        return Err(ResponseError::builder()
            .title("Cannot create those ips".to_string())
            .detail("This network is not set up to independent IPs".to_string())
            .status(StatusCode::FORBIDDEN)
            .build());
    }

    let addrs = network.addresses().unwrap();
    let len = addrs.len();
    let mut transaction = state.transaction().await?;
    for addr in addrs {
        if let Err(e) = transaction.insert(addr).await {
            transaction.rollback().await?;
            return Err(ResponseError::from(e));
        }
    }

    transaction.commit().await?;

    Ok(QueryResult::Insert(len as u64))
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

    Ok(addrs)
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

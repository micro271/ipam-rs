use super::{
    Json, Path, Query, RepositoryType, ResponseDefault, State,
    entries::{
        models::AddrCrateEntry,
        params::{IpNetParamNonOption, PaginationParams, ParamAddrFilter},
    },
    extractors::IsAdministrator,
};
use crate::{
    database::{repository::Repository, transaction::Transaction},
    models::network::{
        Kind, Network, NetworkFilter,
        addresses::{Addr, Addresses, StatusAddr},
    },
    response::ResponseQuery,
};
use axum::http::StatusCode;
use ipnet::IpNet;
use libipam::response_error::ResponseError;
use serde_json::json;
use uuid::Uuid;

const BATCH: usize = 8192;

pub async fn insert(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(new_addr): Json<AddrCrateEntry>,
) -> ResponseDefault<()> {
    Ok(state.insert::<Addresses>(new_addr.into()).await?.into())
}

pub async fn create_all_ip_addresses(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(network_id): Path<Uuid>,
) -> ResponseDefault<()> {
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
        .remove(0);

    if network.kind != Kind::Network {
        return Err(ResponseError::builder()
            .title("Cannot create those ips".to_string())
            .detail("This network is not set up to independent IPs".to_string())
            .status(StatusCode::FORBIDDEN)
            .build());
    }

    let len;

    match network.addresses() {
        Ok(e) if e.len() > BATCH => {
            let addrs = e.batch(BATCH);
            len = addrs.inner_len();

            let mut transaction = state.transaction().await?;
            for addr in addrs {
                if let Err(e) = transaction.insert_many(addr).await {
                    transaction.rollback().await?;
                    return Err(ResponseError::from(e));
                }
            }

            transaction.commit().await?;
        }
        Ok(e) => {
            len = e.len();
            _ = state.insert_many(e.collect::<Vec<_>>()).await?;
        }
        Err(e) => {
            return Err(ResponseError::builder()
                .detail(e.to_string())
                .status(StatusCode::BAD_REQUEST)
                .build());
        }
    }

    let metadata = Some(json!({
        "row_affect": len,
        "status": StatusCode::OK.as_u16(),
        "success": true
    }));

    Ok(ResponseQuery::new(None, metadata, None, StatusCode::OK))
}

pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(network_id): Path<Uuid>,
    Query(IpNetParamNonOption { ip }): Query<IpNetParamNonOption>,
    Json(updater): Json<Addr>,
) -> Result<StatusCode, ResponseError> {
    let mut transaction = state.transaction().await?;

    if updater.network_id.is_some_and(|x| x != network_id) || updater.ip.is_some_and(|x| x != ip) {
        if let Some(id) = updater.network_id {
            let netw = transaction
                .get::<Network>(
                    Addr {
                        network_id: Some(id),
                        ..Default::default()
                    },
                    None,
                    None,
                )
                .await?
                .remove(0);

            if netw.kind != Kind::Network {
                return Err(ResponseError::builder()
                    .detail("This network doesn't split into simple IPs.".to_string())
                    .status(StatusCode::BAD_REQUEST)
                    .build());
            }

            if !netw.subnet.contains(&updater.ip.unwrap_or(ip)) {
                return Err(ResponseError::builder()
                    .detail(format!(
                        "The ip {:?} does't belong to network {:?}",
                        updater.ip.unwrap_or(ip),
                        netw.subnet,
                    ))
                    .build());
            }
        }

        let network_id_to_replace = updater.network_id.or(Some(network_id));
        let ip_to_replace = updater.ip.or(Some(ip));

        let to_replace = Addr {
            network_id: network_id_to_replace,
            ip: ip_to_replace,
            ..Default::default()
        };

        if let Ok(mut addr) = transaction
            .get::<Addresses>(to_replace.clone(), None, None)
            .await
        {
            let addr = addr.remove(0);

            if addr.status != StatusAddr::Unknown {
                return Err(ResponseError::builder()
                    .detail("Tht ip address target is not unknown status".to_string())
                    .status(StatusCode::FORBIDDEN)
                    .build());
            }

            if let Err(e) = transaction.delete::<Addresses, _>(to_replace).await {
                return Err(transaction
                    .rollback()
                    .await
                    .map(|()| ResponseError::from(e))?);
            }
        }

        transaction
            .update::<Addresses, _, _>(
                updater,
                Addr {
                    ip: Some(ip),
                    network_id: Some(network_id),
                    ..Default::default()
                },
            )
            .await?;

        let new_address = Addresses {
            ip,
            network_id,
            status: StatusAddr::Unknown,
            node_id: None,
        };

        transaction.insert(new_address).await?;

        transaction.commit().await?;

        Ok(StatusCode::OK)
    } else {
        let condition = Addr {
            ip: Some(ip),
            network_id: Some(network_id),
            ..Default::default()
        };

        if let Err(e) = transaction
            .update::<Addresses, _, _>(updater, condition)
            .await
        {
            return Err(transaction
                .rollback()
                .await
                .map(|()| ResponseError::from(e))?);
        }

        transaction.commit().await?;

        Ok(StatusCode::OK)
    }
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
) -> ResponseDefault<Vec<Addresses>> {
    let mut addrs = state
        .get::<Addresses>(
            Addr {
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
        addrs.sort_by_key(|x| x.ip);
    }

    Ok(ResponseQuery::new(Some(addrs), None, None, StatusCode::OK))
}

pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(network_id): Path<Uuid>,
    Query(ip): Query<IpNet>,
) -> ResponseDefault<()> {
    let del = state
        .delete::<Addresses>(Addr {
            network_id: Some(network_id),
            ip: Some(ip),
            ..Default::default()
        })
        .await?;

    Ok(del.into())
}

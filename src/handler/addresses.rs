use super::{
    BATCH_SIZE, Json, Path, Query, RepositoryType, ResponseDefault, State,
    entries::{
        models::AddrCrateEntry,
        params::{IpNetParamNonOption, PaginationParams, ParamAddrFilter},
    },
    extractors::IsAdministrator,
};
use crate::{
    database::{
        repository::Repository,
        transaction::{BuilderPgTransaction, Transaction as _},
    },
    models::network::{
        Kind, NetwCondition, Network, UpdateHostCount,
        addresses::{AddrCondition, Addresses, StatusAddr},
    },
    response::ResponseQuery,
};
use axum::http::StatusCode;
use ipnet::IpNet;
use libipam::response_error::ResponseError;
use serde_json::json;
use uuid::Uuid;

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
        .get::<Network>(NetwCondition::p_key(network_id), None, None)
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

    _ = state.heavy_task().acquire().await;

    match network.addresses() {
        Ok(e) if e.len() > BATCH_SIZE => {
            let addrs = e.batch(BATCH_SIZE);
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
    Json(updater): Json<AddrCondition>,
) -> Result<StatusCode, ResponseError> {
    if updater.ip.is_some_and(|x| x != ip) || updater.network_id.is_some_and(|x| x != network_id) {
        let network_target = state
            .get_one::<Network>(NetwCondition::p_key(
                updater.network_id.unwrap_or(network_id),
            ))
            .await?;

        if network_target.kind != Kind::Network
            || network_target.subnet.contains(&updater.ip.unwrap_or(ip))
        {
            return Err(ResponseError::builder()
                .detail("The network cannot be assigned one ip address or the ip isn't belong to the network".to_string())
                .status(StatusCode::BAD_REQUEST)
                .build());
        }

        let mut to_delete = state
            .get_one::<Addresses>(AddrCondition::p_key(
                updater.ip.unwrap_or(ip),
                network_target.id,
            ))
            .await
            .ok();

        if to_delete
            .as_ref()
            .is_some_and(|x| x.status != StatusAddr::Unknown)
        {
            return Err(ResponseError::builder().build());
        }

        let mut transaction = state.transaction().await?;

        if let Err(e) = {
            let to_update = state
                .get_one::<Addresses>(AddrCondition::p_key(ip, network_id))
                .await?;

            if to_update.status != StatusAddr::Unknown && network_target.id != network_id {
                let network_to_increase_free_hc = state
                    .get_one::<Network>(NetwCondition::p_key(network_id))
                    .await?;

                update_host_count(
                    &mut transaction,
                    network_to_increase_free_hc,
                    1,
                    UpdateHostCount::less_used_more_free,
                )
                .await?;

                update_host_count(
                    &mut transaction,
                    network_target,
                    1,
                    UpdateHostCount::less_free_more_used,
                )
                .await?;
            }

            if let Some(addr) = to_delete.as_mut() {
                transaction
                    .delete::<Addresses, _>(AddrCondition::p_key(addr.ip, addr.network_id))
                    .await?;
                addr.ip = ip;
                addr.network_id = network_id;
            }

            let condition_addr_to_update = AddrCondition::p_key(
                updater.ip.unwrap_or(ip),
                updater.network_id.unwrap_or(network_id),
            );

            transaction
                .update::<Addresses, _, _>(updater, condition_addr_to_update)
                .await?;

            if let Some(e) = to_delete {
                transaction.insert(e).await?;
            }

            Result::Ok::<(), ResponseError>(())
        } {
            transaction.rollback().await?;
            return Err(e);
        }

        transaction.commit().await?;
        Ok(StatusCode::OK)
    } else {
        state
            .update::<Addresses, _>(updater, AddrCondition::p_key(ip, network_id))
            .await?;
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
        .delete::<Addresses>(AddrCondition::p_key(ip, network_id))
        .await?;

    Ok(del.into())
}

pub async fn update_host_count<F>(
    transaction: &mut BuilderPgTransaction<'_>,
    mut network: Network,
    n: u32,
    mut action: F,
) -> Result<(), ResponseError>
where
    F: FnMut(&mut UpdateHostCount, u32),
{
    let mut hc = network.update_host_count();
    action(&mut hc, n);

    transaction
        .update::<Network, _, _>(hc, NetwCondition::p_key(network.id))
        .await?;

    while let Some(father) = network.father {
        network = transaction
            .get(NetwCondition::p_key(father), None, None)
            .await?
            .remove(0);

        let mut hc = network.update_host_count();
        action(&mut hc, n);

        transaction
            .update::<Network, _, _>(hc, NetwCondition::p_key(father))
            .await?;
    }

    Ok(())
}

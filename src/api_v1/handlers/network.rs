use crate::{
    database::transaction::Transaction as _,
    models::network::{DefaultValuesNetwork, NetwCondition, UpdateHostCount},
    response::ResponseQuery,
};
use std::net::{Ipv4Addr, Ipv6Addr};

use super::{
    BATCH_SIZE, IsAdministrator, Json, PaginationParams, Path, Query, QueryResult, Repository,
    ResponseDefault, ResponseError, State, StateType, StatusCode, Uuid,
    addresses::update_host_count,
    entries::{self, models::CreateSubnet},
    models,
};

use entries::{models::NetworkCreateEntry, params::ParamNetwork};
use models::network::{Network, UpdateNetwork};
use serde_json::json;

pub async fn create(
    State(state): State<StateType>,
    _: IsAdministrator,
    Json(mut network): Json<NetworkCreateEntry>,
) -> ResponseDefault<()> {
    let net = network.subnet.network();

    match net {
        std::net::IpAddr::V4(ipv4_addr) => {
            if [
                Ipv4Addr::BROADCAST,
                Ipv4Addr::LOCALHOST,
                Ipv4Addr::UNSPECIFIED,
            ]
            .contains(&ipv4_addr)
            {
                return Err(ResponseError::builder()
                    .title("Invalid network".to_string())
                    .detail(format!("We cannot create the network {ipv4_addr}"))
                    .build());
            }
        }
        std::net::IpAddr::V6(ipv6_addr) => {
            if [Ipv6Addr::LOCALHOST, Ipv6Addr::UNSPECIFIED].contains(&ipv6_addr) {
                return Err(ResponseError::builder()
                    .title("Invalid network".to_string())
                    .detail(format!("We cannot create the network {ipv6_addr}"))
                    .build());
            }
        }
    }

    network.subnet = ipnet::IpNet::new(net, network.subnet.prefix_len()).unwrap();
    Ok(state.insert::<Network>(network.into()).await?.into())
}

pub async fn get(
    State(state): State<StateType>,
    Query(param): Query<ParamNetwork>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> ResponseDefault<Vec<Network>> {
    let data = state.get::<Network>(param, limit, offset).await?;

    let metadata = Some(json!({
        "length": data.len(),
        "success": true,
        "status": StatusCode::OK.as_u16(),
    }));

    Ok(ResponseQuery::new(
        Some(data),
        metadata,
        None,
        StatusCode::OK,
    ))
}

pub async fn update(
    State(state): State<StateType>,
    _: IsAdministrator,
    Path(id): Path<Uuid>,
    Json(updater): Json<UpdateNetwork>,
) -> ResponseDefault<()> {
    if updater.network.is_some() {
        let old = state.get_one::<Network>(NetwCondition::p_key(id)).await?;

        if old.children != 0 {
            tracing::debug!("The network {:?} have subnets", old.subnet);
            return Err(ResponseError::builder()
                .detail("the network have child".into())
                .status(StatusCode::BAD_REQUEST)
                .build());
        } else if (old.used.as_i32() + old.free.as_i32()) != old.free.as_i32() {
            tracing::debug!("The network {:?} have devices", old.subnet);
            return Err(ResponseError::builder()
                .detail("the network have devices".into())
                .status(StatusCode::BAD_REQUEST)
                .build());
        }
    }

    let resp = state
        .update::<Network, _>(updater, NetwCondition::p_key(id))
        .await?;

    Ok(resp.into())
}

pub async fn delete(
    State(state): State<StateType>,
    _: IsAdministrator,
    Path(id): Path<Uuid>,
) -> ResponseDefault<()> {
    tracing::debug!("delete one network: {}", id);

    let for_delete = state.get_one::<Network>(NetwCondition::p_key(id)).await?;

    let mut transaction = state.transaction().await?;

    if let Some(father) = for_delete.father {
        let res = {
            let resp = transaction
                .delete::<Network, _>(NetwCondition::father(father))
                .await?;

            let father = state
                .get_one::<Network>(NetwCondition::p_key(father))
                .await?;

            let mut hc = father.update_host_count();
            hc.new_calculate();

            transaction
                .update::<Network, _, _>(hc, NetwCondition::p_key(father.id))
                .await?;

            Result::Ok::<QueryResult, ResponseError>(resp)
        };

        match res {
            Ok(e) => Ok(e.into()),
            Err(e) => {
                transaction.rollback().await?;
                Err(e)
            }
        }
    } else {
        let resp = state.delete::<Network>(NetwCondition::p_key(id)).await?;

        Ok(resp.into())
    }
}

pub async fn subnetting(
    State(state): State<StateType>,
    _: IsAdministrator,
    Path(father): Path<Uuid>,
    Json(CreateSubnet {
        prefix,
        status,
        kind,
        description,
    }): Json<CreateSubnet>,
) -> ResponseDefault<()> {
    let father = state
        .get_one::<Network>(NetwCondition::p_key(father))
        .await?;

    let mut subnet = father.subnets(prefix).map_err(|x| {
        ResponseError::builder()
            .detail(x.to_string())
            .status(StatusCode::BAD_REQUEST)
    })?;

    subnet.set_default_values(DefaultValuesNetwork::new(
        father.id,
        status,
        kind,
        description,
    ));

    let _permit = state.heavy_task().acquire().await;

    let mut transaction = state.transaction().await?;
    let len = subnet.len();

    if let Err(e) = update_host_count(
        &mut transaction,
        father,
        len.try_into().unwrap(),
        UpdateHostCount::less_free_more_used,
    )
    .await
    {
        transaction.rollback().await?;
        return Err(e);
    }

    if len >= BATCH_SIZE {
        let subnet = subnet.batch(BATCH_SIZE);

        for net in subnet {
            if let Err(e) = transaction.insert_many(net).await {
                transaction.rollback().await?;
                return Err(ResponseError::from(e));
            }
        }
    } else {
        let subnet = subnet.collect::<Vec<_>>();

        if let Err(e) = transaction.insert_many(subnet).await {
            transaction.rollback().await?;
            return Err(ResponseError::from(e));
        }
    }

    Ok(QueryResult::new(10).into())
}

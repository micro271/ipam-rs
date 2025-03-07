use crate::database::transaction::Transaction as _;
use std::net::{Ipv4Addr, Ipv6Addr};

use super::{
    HashMap, IsAdministrator, Json, Level, PaginationParams, Path, Query, QueryResult, Repository,
    RepositoryType, ResponseError, State, StatusCode, Uuid, entries, instrument, models,
};

use entries::{
    models::NetworkCreateEntry,
    params::{ParamNetwork, Subnet},
};
use models::network::{Network, To, UpdateNetwork};

#[instrument(level = Level::INFO)]
pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(network): Json<NetworkCreateEntry>,
) -> Result<QueryResult<Network>, ResponseError> {
    let net = network.network.network();

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

    Ok(state.insert::<Network>(network.into()).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(param): Query<ParamNetwork>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Network>, ResponseError> {
    Ok(state.get::<Network>(param, limit, offset).await?.into())
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(id): Path<Uuid>,
    Json(updater): Json<UpdateNetwork>,
) -> Result<QueryResult<Network>, ResponseError> {
    if updater.network.is_some() {
        let old = state
            .get::<Network>(Some([("id", id.into())].into()), None, None)
            .await?
            .remove(0);

        if old.children != 0 {
            tracing::debug!("The network {:?} have subnets", old.network);
            return Err(ResponseError::builder()
                .detail("the network have child".into())
                .status(StatusCode::BAD_REQUEST)
                .build());
        } else if old.available != old.free {
            tracing::debug!("The network {:?} have devices", old.network);
            return Err(ResponseError::builder()
                .detail("the network have devices".into())
                .status(StatusCode::BAD_REQUEST)
                .build());
        }
    }
    Ok(state
        .update::<Network, _>(updater, Some([("id", id.into())].into()))
        .await?)
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(id): Path<Uuid>,
) -> Result<QueryResult<Network>, ResponseError> {
    tracing::debug!("delete one network: {}", id);

    Ok(state
        .delete::<Network>(Some([("id", id.into())].into()))
        .await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn subnetting(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(Subnet { father, prefix }): Query<Subnet>,
) -> Result<QueryResult<Network>, ResponseError> {
    let father = state
        .get::<Network>(Some([("id", father.into())].into()), None, None)
        .await?
        .remove(0);

    if father.use_to == To::Device {
        return Err(ResponseError::builder()
            .detail("The father is to device".to_string())
            .build());
    }

    let networks = father.subnets(prefix).map_err(|x| {
        ResponseError::builder()
            .detail(x.to_string())
            .status(StatusCode::BAD_REQUEST)
            .build()
    })?;

    let mut state = state.transaction().await?;
    let len = networks.len();

    for network in networks {
        let mut new_network = Network::from(network);
        new_network.father = Some(father.id);

        if let Err(e) = state.insert(new_network).await {
            state.rollback().await?;
            return Err(ResponseError::from(e));
        }
    }

    state
        .update::<Network, _, _>(
            HashMap::from([("children", i32::try_from(len).unwrap_or_default().into())]), /* TODO: we've updated the free and available ips */
            Some([("id", father.id.into())].into()),
        )
        .await?;

    state.commit().await?;

    Ok(QueryResult::Insert(len as u64))
}

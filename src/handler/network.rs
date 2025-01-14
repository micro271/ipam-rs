use std::{net::IpAddr, str::FromStr};

use super::RepositoryType;
use super::*;
use crate::{
    database::{
        repository::{QueryResult, TypeTable},
        transaction::Transaction,
    },
    models::{device::Device, network::*},
};
use entries::{models::NetworkCreateEntry, params::Subnet};

pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(network): Json<NetworkCreateEntry>,
) -> Result<QueryResult<Network>, ResponseError> {
    let net = network.network.network();

    if net == IpAddr::from_str("0.0.0.0").unwrap() || net == IpAddr::from_str("::").unwrap() {
        return Err(ResponseError::builder()
            .detail(format!("You cannot create the ip {:?}", network.network))
            .build());
    }

    Ok(state.insert::<Network>(network.into()).await?)
}

pub async fn get(
    State(state): State<RepositoryType>,
    Path(id): Path<Option<Uuid>>,
) -> Result<QueryResult<Network>, ResponseError> {
    Ok(state
        .get::<Network>(id.map(|x| HashMap::from([("id", x.into())])))
        .await?
        .into())
}

pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(id): Query<Uuid>,
    Json(updater): Json<UpdateNetwork>,
) -> Result<QueryResult<Network>, ResponseError> {
    let network = state
        .get::<Network>(Some(HashMap::from([("id", id.into())])))
        .await?
        .remove(0);

    if updater.network.is_some()
        && (state
            .get::<Device>(Some(HashMap::from([("network_id", id.into())])))
            .await
            .is_ok()
            || network.children != 0)
    {
        Err(ResponseError::builder()
            .detail("The network have elements".to_string())
            .build())
    } else {
        Ok(state
            .update(updater, Some(HashMap::from([("id", id.into())])))
            .await?)
    }
}

pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(id): Path<Uuid>,
) -> Result<QueryResult<Network>, ResponseError> {
    Ok(state
        .delete::<Network>(Some(HashMap::from([("id", id.into())])))
        .await?)
}

pub async fn subnetting(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(Subnet { father, prefix }): Query<Subnet>,
) -> Result<QueryResult<Network>, ResponseError> {
    let father = state
        .get::<Network>(Some(HashMap::from([("id", father.into())])))
        .await?
        .remove(0);

    let networks = libipam::ipam_services::subnetting(father.network, prefix)
        .map_err(|x| ResponseError::builder().detail(x.to_string()).build())?;

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
        .update::<Network, _>(
            HashMap::from([("children", TypeTable::from(len as i32))]),
            Some(HashMap::from([("id", father.id.into())])),
        )
        .await?;

    state.commit().await?;

    Ok(QueryResult::Insert(len as u64))
}

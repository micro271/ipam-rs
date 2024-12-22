use std::net::IpAddr;

use super::RepositoryType;
use super::*;
use crate::{
    database::{repository::QueryResult, transaction::Transaction},
    models::network::*,
};
use models::device::Device;
use params::network::QueryNetwork;

pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(netw): Json<models_data_entry::Network>,
) -> Result<QueryResult<Network>, ResponseError> {
    let state = state.lock().await;

    Ok(state.insert::<Network>(vec![netw.into()]).await?)
}

pub async fn get(
    State(state): State<RepositoryType>,
    Query(param): Query<QueryNetwork>,
) -> Result<QueryResult<Network>, ResponseError> {
    let state = state.lock().await;

    Ok(state.get::<Network>(param.get_condition()).await?.into())
}

pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(id): Query<Uuid>,
    Json(updater): Json<UpdateNetwork>,
) -> Result<QueryResult<Network>, ResponseError> {
    let state = state.lock().await;
    let network_current = state
        .get::<Network>(Some(HashMap::from([("id", id.into())])))
        .await?
        .remove(0);

    let mut transaction = state.transaction().await.unwrap();
    let network = updater.network.clone();

    transaction
        .update::<Network, _>(updater, Some(HashMap::from([("id", id.into())])))
        .await;

    if network.is_some_and(|x| x != network_current.network) {
        let tmp = network.unwrap();
        let mut devices = state
            .get::<Device>(Some(HashMap::from([(
                "network_id",
                network_current.network.into(),
            )])))
            .await?;
        devices.sort_by_key(|x| x.ip);
        let host = tmp.hosts().map(|x| x).collect::<Vec<IpAddr>>();
        for (pos, h) in host.into_iter().enumerate() {
            transaction
                .update::<Device, _>(h, Some(HashMap::from([("ip", devices[pos].ip.into())])))
                .await;
        }
    }

    Ok(QueryResult::Update(50))
}

pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(id): Query<Uuid>,
) -> Result<QueryResult<Network>, ResponseError> {
    let state = state.lock().await;

    Ok(state
        .delete::<Network>(Some(HashMap::from([("id", id.into())])))
        .await?)
}

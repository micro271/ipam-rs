use super::RepositoryType;
use super::*;
use crate::{
    database::{repository::QueryResult, transaction::Transaction},
    models::{device::Device, network::*},
};
use entries::{models::NetworkCreateEntry, params::Subnet};
use libipam::type_net::host_count::HostCount;

pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(network): Json<NetworkCreateEntry>,
) -> Result<QueryResult<Network>, ResponseError> {
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
    
    if updater.network.is_some() && state.get::<Device>(Some(HashMap::from([("network_id",id.into())]))).await.is_ok() {
        Err(ResponseError::builder().detail("The network have devices".to_string()).build())
    } else {
        Ok(state.update(updater, Some(HashMap::from([("id", id.into())]))).await?)
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

pub async fn subnetting(State(state): State<RepositoryType>, _: IsAdministrator, Query(Subnet { father, prefix }): Query<Subnet>) -> Result<QueryResult<Network>, ResponseError> {
    let father = state.get::<Network>(Some(HashMap::from([("id", father.into())]))).await?.remove(0);

    let networks = libipam::ipam_services::subnetting(father.network, prefix).map_err(|x| ResponseError::builder().detail(x.to_string()).build())?;

    let mut state = state.transaction().await?;
    let len = networks.len();

    for network in networks {
        let new_network = Network {
            network,
            id: uuid::Uuid::new_v4(),
            available: HostCount::new((&network).into()),
            used: 0.into(),
            free: HostCount::new((&network).into()),
            vlan: None,
            description: None,
            father: Some(father.id),
        };

        if let Err(e) = state.insert(new_network).await {
            state.rollback().await?;
            return Err(ResponseError::from(e));
        }
    }
    state.commit().await?;
    
    Ok(QueryResult::Insert(len as u64))

}

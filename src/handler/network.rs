use super::RepositoryType;
use super::*;
use crate::{
    database::repository::QueryResult,
    models::{device::Device, network::*},
};
use entries::models::NetworkCreateEntry;

pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(netw): Json<NetworkCreateEntry>,
) -> Result<QueryResult<Network>, ResponseError> {
    Ok(state.insert::<Network>(netw.into()).await?)
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

use super::*;
use params::network::QueryNetwork;
use super::RepositoryType;
use crate::{database::repository::QueryResult, models::network::*};

pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(netw): Json<models_data_entry::Network>,
) -> Result<QueryResult<Network>, ResponseError> {
    let state = state.lock().await;

    Ok(
        state.insert::<Network>(vec![netw.into()]).await?
    )
}

pub async fn get(
    State(state): State<RepositoryType>,
    Query(param): Query<QueryNetwork>,
) -> Result<QueryResult<Network>, ResponseError> {
    let state = state.lock().await;

    Ok(
        state.get::<Network>(param.get_condition()).await?.into()
    )
}

pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(id): Query<Uuid>,
    Json(updater): Json<UpdateNetwork>,
) -> Result<QueryResult<Network>, ResponseError> {
    let state = state.lock().await;

    Ok(
        state.update::<'_, Network, _>(updater, Some(HashMap::from([("id", id.into())]))).await?
    )
}

pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(id): Query<Uuid>,
) -> Result<QueryResult<Network>, ResponseError> {
    let state = state.lock().await;

    Ok(
        state.delete::<Network>(Some(HashMap::from([("id", id.into())]))).await?
    )
}

use super::RepositoryType;
use super::*;
use crate::{
    database::{repository::QueryResult, transaction::BuilderPgTransaction},
    models::network::*,
};
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
    let tr = state.begin().await.unwrap();
    let state = Arc::new(Mutex::new(tr));
    let tmp = BuilderPgTransaction::new(state.clone()).await;
    println!("{:?}", Arc::strong_count(&state));
    let tr = Arc::try_unwrap(state).unwrap().into_inner();
    tr.commit().await.unwrap();
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

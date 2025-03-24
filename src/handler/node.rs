use super::{
    IntoResponse, IsAdministrator, Json, Level, PaginationParams, Path, Query, Repository,
    RepositoryType, ResponseError, State, StatusCode, Uuid, entries, instrument,
};
use crate::{
    database::{repository::QueryResult, transaction::Transaction},
    models::{
        network::{Kind, Network},
        node::{Node, UpdateNode},
    },
};
use entries::{
    models::NodeCreateEntry,
    params::{ParamsDevice, ParamsDeviceStrict},
};
use libipam::services::ipam::Ping;

#[instrument(level = Level::DEBUG)]
pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(node): Json<NodeCreateEntry>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.insert::<Node>(node.into()).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn create_all_devices(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(network_id): Path<Uuid>,
) -> Result<QueryResult<Node>, ResponseError> {
    let network = state
        .get::<Network>(Some([("id", network_id.into())].into()), None, None)
        .await?
        .take_data()
        .unwrap()
        .remove(0);

    if network.kind == Kind::Pool {
        return Err(ResponseError::builder()
            .detail(
                "The kind of subnet is not a Network, so we cannot create all nodes because one pool of IPs is for one node".to_string(),
            )
            .status(StatusCode::BAD_REQUEST)
            .build());
    }

    let addrs = network.addresses().map_err(|x| {
        ResponseError::builder()
            .detail(x.to_string())
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .build()
    })?;

    let mut transaction = state.transaction().await?;
    let len = addrs.len();
    for addr in addrs {
        if let Err(e) = transaction.insert(addr).await {
            return Err(transaction
                .rollback()
                .await
                .map(|_| ResponseError::from(e))?);
        }
    }
    transaction.commit().await?;

    Ok(QueryResult::Insert(len as u64))
}

#[instrument(level = Level::DEBUG)]
pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
    Json(new): Json<UpdateNode>,
) -> Result<StatusCode, ResponseError> {
    todo!()
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(params): Query<ParamsDevice>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Node>, ResponseError> {
    Ok(state.get::<Node>(params, limit, offset).await?.into())
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.delete::<Node>(param).await?)
}

#[instrument(level = Level::INFO)]
pub async fn ping(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(condition): Query<ParamsDeviceStrict>,
) -> Result<Ping, ResponseError> {
    todo!()
}

use super::*;
use crate::{
    database::{repository::QueryResult, transaction::Transaction},
    models::{device::*, network::Network},
};
use entries::{
    models,
    params::{GetMapParams, ParamsDevice, ParamsDeviceStrict},
};

pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(device): Json<models::Device>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.insert::<Device>(device.into()).await?)
}

pub async fn create_all_devices(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(network_id): Path<Uuid>,
) -> Result<QueryResult<Device>, ResponseError> {
    let network = state
        .get::<Network>(Some(HashMap::from([("id", network_id.into())])))
        .await?
        .remove(0);

    let devices =  models::create_all_devices(network.network, network_id).map_err(|x|{
        ResponseError::builder().detail(x.to_string()).build()
    })?;

    let mut transaction = state.transaction().await?;
    let len = devices.len();
    for device in devices {
        if let Err(e) = transaction.insert(device).await {
            transaction.rollback().await?;
            return Err(ResponseError::builder().detail(e.to_string()).title(format!("Device create error")).status(StatusCode::INTERNAL_SERVER_ERROR).build())
        }
    }
    transaction.commit().await?;

    Ok(QueryResult::Insert(len as u64))
}

pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
    Json(new): Json<UpdateDevice>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.update::<Device, _>(new, param.get_pairs()).await?)
}

pub async fn get(
    State(state): State<RepositoryType>,
    Query(params): Query<ParamsDevice>,
) -> Result<QueryResult<Device>, ResponseError> {
    let mut device = state.get::<Device>(params.get_pairs()).await?;

    device.sort_by_key(|x| x.ip );

    Ok(device.into())
}

pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.delete::<Device>(param.get_pairs()).await?)
}

use super::*;
use crate::{
    database::repository::QueryResult,
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
    Ok(state.insert::<Device>(vec![device.into()]).await?)
}

pub async fn create_all_devices(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(network_id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    let network = state
        .get::<Network>(Some(HashMap::from([("id", network_id.into())])))
        .await?
        .remove(0);

    match models::create_all_devices(network.network, network_id) {
        Ok(e) => Ok(state.insert::<Device>(e).await?),
        _ => Err(ResponseError::builder()
            .status(StatusCode::NO_CONTENT)
            .build()),
    }
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
    let device = state.get::<Device>(params.get_pairs()).await?;

    Ok(device.into())
}

pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.delete::<Device>(param.get_pairs()).await?)
}

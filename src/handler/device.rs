use super::*;
use crate::{
    database::{repository::QueryResult, transaction::Transaction},
    models::{
        device::*,
        network::{Network, To},
    },
};
use entries::{
    models::DeviceCreateEntry,
    params::{ParamsDevice, ParamsDeviceStrict},
};

#[instrument(level = Level::DEBUG)]
pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(device): Json<DeviceCreateEntry>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.insert::<Device>(device.into()).await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn create_all_devices(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Path(network_id): Path<Uuid>,
) -> Result<QueryResult<Device>, ResponseError> {
    let network = state
        .get::<Network>(Some([("id", network_id.into())].into()), None, None)
        .await?
        .remove(0);

    if network.to != To::Device {
        return Err(ResponseError::builder()
            .detail("The network is designed for devices".to_string())
            .status(StatusCode::BAD_REQUEST)
            .build());
    }

    let devices = network
        .devices()
        .map_err(|x| ResponseError::builder().detail(x.to_string()).build())?;

    let mut transaction = state.transaction().await?;
    let len = devices.len();
    for device in devices {
        if let Err(e) = transaction.insert(device).await {
            return Err(transaction.rollback().await.map(|_| {
                ResponseError::builder()
                    .detail(e.to_string())
                    .title("Device create error".to_string())
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .build()
            })?);
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
    Json(new): Json<UpdateDevice>,
) -> Result<StatusCode, ResponseError> {
    if new
        .status
        .is_some_and(|x| x != Status::Unknown && x != Status::Reserved)
    {
        return Err(ResponseError::builder()
            .detail(format!(
                "The status cannot change to {:?}",
                new.status.unwrap()
            ))
            .build());
    }

    if new.ip.is_some_and(|x| x == param.ip)
        && new.network_id.is_some_and(|x| x == param.network_id)
    {
        return Err(ResponseError::builder()
            .detail("The new ip and network are the same".to_string())
            .status(StatusCode::BAD_REQUEST)
            .build());
    } else if new.ip.is_some() || new.network_id.is_some() {
        let Device {
            ip,
            network_id,
            status,
            ..
        } = state
            .get::<Device>(
                Some(
                    [
                        (
                            "network_id",
                            new.network_id.unwrap_or(param.network_id).into(),
                        ),
                        ("ip", new.ip.unwrap_or(param.ip).into()),
                    ]
                    .into(),
                ),
                None,
                None,
            )
            .await?
            .remove(0);

        if status != Status::Unknown {
            return Err(ResponseError::builder()
                .detail("The device to replace isn't unknown".to_string())
                .status(StatusCode::FORBIDDEN)
                .build());
        }

        let mut tr = state.transaction().await?;

        if let Err(e) = {
            tr.delete::<Device, _>(Some(
                [("network_id", network_id.into()), ("ip", ip.into())].into(),
            ))
            .await?;

            tr.update::<Device, _, _>(new, param).await?;

            tr.insert::<Device>((param.ip, param.network_id).into())
                .await?;

            Ok::<(), ResponseError>(())
        } {
            Err(tr.rollback().await.map(|_| e)?)
        } else {
            Ok(tr.commit().await.map(|_| StatusCode::OK)?)
        }
    } else {
        Ok(state
            .update::<Device, _>(new, param)
            .await
            .map(|_| StatusCode::OK)?)
    }
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(params): Query<ParamsDevice>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Device>, ResponseError> {
    let mut device = state.get::<Device>(params, limit, offset).await?;

    device.sort_by_key(|x| x.ip);

    Ok(device.into())
}

#[instrument(level = Level::INFO)]
pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.delete::<Device>(param).await?)
}

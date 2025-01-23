use super::*;
use crate::{
    database::{repository::QueryResult, transaction::Transaction},
    models::{device::*, network::Network},
};
use entries::{
    models::{self, DeviceCreateEntry},
    params::{ParamsDevice, ParamsDeviceStrict},
};

pub async fn create(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Json(device): Json<DeviceCreateEntry>,
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

    let devices = models::create_all_devices(network.network, network_id)
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

pub async fn update(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
    Json(mut new): Json<UpdateDevice>,
) -> Result<StatusCode, ResponseError> {
    let network = state
        .get::<Network>(Some(HashMap::from([(
            "id",
            new.network_id.unwrap_or(param.network_id).into(),
        )])))
        .await?
        .remove(0);

    if new.ip.is_some_and(|x| x == param.ip) {
        new.ip = None;
    }

    if network.id == param.network_id {
        new.network_id = None;
    }

    if new.ip.is_some() || new.network_id.is_some() {
        let dev = state
            .get::<Device>(Some(HashMap::from([(
                "network_id",
                network.network.into(),
            )])))
            .await?
            .remove(0);

        if dev.status == Status::Unknown {
            let mut tr = state.transaction().await?;

            if let Err(e) = async {
                tr.delete::<Device>(Some(HashMap::from([(
                    "network_id",
                    network.network.into(),
                )])))
                .await?;

                tr.update::<Device, _>(new, param.get_pairs()).await?;

                Ok::<(), ResponseError>(())
            }
            .await
            {
                Err(tr.rollback().await.map(|_| e)?)
            } else {
                Ok(tr.commit().await.map(|_| StatusCode::OK)?)
            }
        } else {
            Err(ResponseError::builder()
                .detail(format!("The device {:?} is used", dev))
                .build())
        }
    } else {
        Ok(state
            .update::<Device, _>(new, param.get_pairs())
            .await
            .map(|_| StatusCode::OK)?)
    }
}

pub async fn get(
    State(state): State<RepositoryType>,
    Query(params): Query<ParamsDevice>,
    Query(_pagination): Query<PaginationParams>
) -> Result<QueryResult<Device>, ResponseError> {
    let mut device = state.get::<Device>(params.get_pairs()).await?;

    device.sort_by_key(|x| x.ip);

    Ok(device.into())
}

pub async fn delete(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(param): Query<ParamsDeviceStrict>,
) -> Result<impl IntoResponse, ResponseError> {
    Ok(state.delete::<Device>(param.get_pairs()).await?)
}

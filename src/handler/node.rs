use std::collections::HashMap;

use super::{
    IntoResponse, IsAdministrator, Json, Level, PaginationParams, Path, Query, Repository,
    RepositoryType, ResponseError, State, StatusCode, Uuid, entries, instrument,
};
use crate::{
    database::{
        repository::{QueryResult, TypeTable},
        transaction::Transaction,
    },
    models::{
        device::{Node, Status, UpdateNode},
        network::{Network, Target},
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
        .remove(0);

    if network.target != Target::Device {
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
            transaction.rollback().await?;

            return Err(ResponseError::from(e));
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
        let Node {
            ip,
            network_id,
            status,
            ..
        } = state
            .get::<Node>(
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
            tr.delete::<Node, _>(Some(
                [("network_id", network_id.into()), ("ip", ip.into())].into(),
            ))
            .await?;

            tr.update::<Node, _, _>(new, param).await?;

            tr.insert::<Node>((param.ip, param.network_id).into())
                .await?;

            Ok::<(), ResponseError>(())
        } {
            Err(tr.rollback().await.map(|()| e)?)
        } else {
            Ok(tr.commit().await.map(|()| StatusCode::OK)?)
        }
    } else {
        Ok(state
            .update::<Node, _>(new, param)
            .await
            .map(|_| StatusCode::OK)?)
    }
}

#[instrument(level = Level::DEBUG)]
pub async fn get(
    State(state): State<RepositoryType>,
    Query(params): Query<ParamsDevice>,
    Query(PaginationParams { offset, limit }): Query<PaginationParams>,
) -> Result<QueryResult<Node>, ResponseError> {
    let mut device = state.get::<Node>(params, limit, offset).await?;

    device.sort_by_key(|x| x.ip);

    Ok(device.into())
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
pub async fn reserved(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(ParamsDeviceStrict { ip, network_id }): Query<ParamsDeviceStrict>,
) -> Result<impl IntoResponse, ResponseError> {
    let condition: HashMap<&str, TypeTable> =
        [("ip", ip.into()), ("network_id", network_id.into())].into();

    let dev = state
        .get::<Node>(Some(condition.clone()), None, None)
        .await?
        .remove(0);

    if dev.status != Status::Unknown {
        return Err(ResponseError::builder()
            .detail("To change the status to reserved, the device should be unknown".to_string())
            .title("The devices isn't unknown".to_string())
            .build());
    }

    Ok(state
        .update::<Node, _>(
            UpdateNode {
                status: Some(Status::Reserved),
                ..Default::default()
            },
            Some(condition),
        )
        .await?)
}

#[instrument(level = Level::INFO)]
pub async fn unreserved(
    State(state): State<RepositoryType>,
    Query(condition): Query<ParamsDeviceStrict>,
    _: IsAdministrator,
) -> Result<QueryResult<Node>, ResponseError> {
    Ok(state
        .update(
            UpdateNode {
                status: Some(Status::Unknown),
                ..Default::default()
            },
            condition,
        )
        .await?)
}

#[instrument(level = Level::INFO)]
pub async fn ping(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(condition): Query<ParamsDeviceStrict>,
) -> Result<Ping, ResponseError> {
    let dev = state.get::<Node>(condition, None, None).await?.remove(0);

    let ping = libipam::services::ipam::ping(condition.ip, 50).await;

    if ping == Ping::Fail && dev.status == Status::Online {
        state
            .update::<Node, _>(
                UpdateNode {
                    status: Some(Status::Offline),
                    ..Default::default()
                },
                condition,
            )
            .await?;
    } else if ping == Ping::Pong {
        state
            .update::<Node, _>(
                UpdateNode {
                    status: Some(Status::Online),
                    ..Default::default()
                },
                condition,
            )
            .await?;
    }

    Ok(ping)
}

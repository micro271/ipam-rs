use super::{
    IntoResponse, IsAdministrator, Json, Level, PaginationParams, Path, Query, Repository,
    RepositoryType, ResponseError, State, StatusCode, Uuid, entries, instrument,
};
use crate::{
    database::{repository::QueryResult, transaction::Transaction},
    models::{
        network::{Network, Target},
        node::{Node, StatusNode, UpdateNode},
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

    if network.target != Target::Node {
        return Err(ResponseError::builder()
            .detail("The network is designed for nodes".to_string())
            .status(StatusCode::BAD_REQUEST)
            .build());
    }

    let nodes = network.nodes().map_err(|x| {
        ResponseError::builder()
            .detail(x.to_string())
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .build()
    })?;

    let mut transaction = state.transaction().await?;
    let len = nodes.len();
    for node in nodes {
        if let Err(e) = transaction.insert(node).await {
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
    Json(mut new): Json<UpdateNode>,
) -> Result<StatusCode, ResponseError> {
    if new
        .status
        .is_some_and(|x| [StatusNode::Online, StatusNode::Offline].contains(&x))
    {
        return Err(ResponseError::builder()
            .title("Status not valid".to_string())
            .detail("You cann change the status of a node if the new state is Reachable, Reserved or Unknown".to_string())
            .status(StatusCode::BAD_REQUEST)
            .build());
    }

    if new.ip.is_some_and(|x| x == param.ip)
        && new.network_id.is_some_and(|x| x == param.network_id)
    {
        new.ip = None;
        new.network_id = None;
    }

    if new.ip.is_some() || new.network_id.is_some() {
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

        if status != StatusNode::Unknown {
            return Err(ResponseError::builder()
                .detail("The node to replace isn't unknown".to_string())
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

            Ok(())
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
pub async fn ping(
    State(state): State<RepositoryType>,
    _: IsAdministrator,
    Query(condition): Query<ParamsDeviceStrict>,
) -> Result<Ping, ResponseError> {
    let dev = state.get::<Node>(condition, None, None).await?.remove(0);

    let ping = libipam::services::ipam::ping(condition.ip, 50).await;

    if ping == Ping::Fail && dev.status == StatusNode::Online {
        state
            .update::<Node, _>(
                UpdateNode {
                    status: Some(StatusNode::Offline),
                    ..Default::default()
                },
                condition,
            )
            .await?;
    } else if ping == Ping::Pong {
        state
            .update::<Node, _>(
                UpdateNode {
                    status: Some(StatusNode::Online),
                    ..Default::default()
                },
                condition,
            )
            .await?;
    }

    Ok(ping)
}

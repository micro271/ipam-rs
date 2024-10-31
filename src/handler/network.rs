use super::*;


pub async fn create(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Json(netw): Json<models_data_entry::Network>,
) -> Result<impl IntoResponse, ResponseError> {
    if role != Role::Admin {
        return Err(ResponseError::Unauthorized);
    }

    let state = state.lock().await;

    Ok(state.insert::<Network>(vec![netw.into()]).await?)
}

pub async fn get_one(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    let state = state.lock().await;
    let condition = HashMap::from([("id", id.into())]);

    let network = state.get::<Network>(Some(condition)).await?;

    Ok(Json(json!({
        "device": network.first()
    })))
}

pub async fn update(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Path(id): Path<Uuid>,
    Json(network): Json<UpdateNetwork>,
) -> Result<impl IntoResponse, ResponseError> {
    if role != Role::Admin {
        return Err(ResponseError::Unauthorized);
    }

    let state = state.lock().await;

    // Now: Delete all devices that belong to the network
    // Soon: Update all devices
    //     * Only if the prefix of the new network is bigger than or smaller than the current network

    let tmp = state
        .update::<Network, _>(network, Some(HashMap::from([("id", id.into())])))
        .await?;
    match state
        .delete::<Device>(Some(HashMap::from([("network_id", id.into())])))
        .await
    {
        Ok(e) => Ok(Json(json!({
            "update": {
                "row_affect": tmp.unwrap(),
            },
            "delete": {
                "row_affect": e.unwrap()
            }
        }))),
        Err(e) => Err(ResponseError::Custom {
            body: json!({
                "update": {
                    "rows_affect": tmp.unwrap()
                },
                "delete": {
                    "rows_affect": ResponseError::from(e).to_string()
                }
            })
            .to_string(),
            status: StatusCode::OK,
        }),
    }
}

pub async fn get_all(
    State(state): State<RepositoryType>,
) -> Result<impl IntoResponse, ResponseError> {
    let state = state.lock().await;

    let networks = state.get::<Network>(None).await?;

    Ok(Json(json!({
        "networks": networks
    })))
}

pub async fn delete(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    if role != Role::Admin {
        return Err(ResponseError::Unauthorized);
    }

    let state = state.lock().await;

    Ok(state
        .delete::<Network>(Some(HashMap::from([("id", id.into())])))
        .await?)
}

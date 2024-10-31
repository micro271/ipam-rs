use super::*;
use std::net::IpAddr;

pub async fn create(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Json(device): Json<models_data_entry::Device>,
) -> Result<impl IntoResponse, ResponseError> {
    if role != Role::Admin {
        return Err(ResponseError::Unauthorized);
    }

    let state = state.lock().await;

    Ok(state.insert::<Device>(vec![device.into()]).await?)
}

pub async fn create_all_devices(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Path(network_id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    if role != Role::Admin {
        return Err(ResponseError::Unauthorized);
    }

    let state = state.lock().await;
    let network = state
        .get::<Network>(Some(HashMap::from([("id", network_id.into())])))
        .await?
        .remove(0);

    match models_data_entry::create_all_devices(network.network, network_id) {
        Some(e) => Ok(state.insert::<Device>(e).await?),
        None => Err(ResponseError::StatusCode(StatusCode::NO_CONTENT)),
    }
}

pub async fn get_all(
    State(state): State<RepositoryType>,
    Path(network_id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    let state = state.lock().await;
    let condition = HashMap::from([("network_id", network_id.into())]);
    let devices = state.get::<Device>(Some(condition)).await?;

    Ok(Json(json!({
        "devices": devices
    })))
}

pub async fn update(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Query((ip, network_id)): Query<(IpAddr, Uuid)>,
    Json(device): Json<UpdateDevice>,
) -> Result<impl IntoResponse, ResponseError> {
    if role != Role::Admin {
        return Err(ResponseError::Unauthorized);
    }
    let state = state.lock().await;

    if device.network_id.is_some() || device.ip.is_some() {

        let ip_to_delete: IpAddr;

        let netw_new = state
            .get::<Network>(Some(HashMap::from([(
                "id",
                match device.network_id {
                    Some(e) => e.into(),
                    None => network_id.into(),
                },
            )])))
            .await?
            .remove(0);
    
        if let Some(ip) = device.ip {
    
            if !netw_new.network.contains(&ip) {
                return Err(ResponseError::StatusCode(StatusCode::BAD_REQUEST));
            }
            ip_to_delete = ip;
            
        } else {

            if !netw_new.network.contains(&ip) {
                return Err(ResponseError::StatusCode(StatusCode::CONFLICT));
            }
            ip_to_delete = ip;
        }
        
        state.delete::<Device>(Some(HashMap::from([("ip", ip_to_delete.into()), ("network_id", netw_new.id.into())]))).await?;
    } 

    Ok(state
        .update::<Device, _>(
            device,
            Some(HashMap::from([
                ("ip", ip.into()),
                ("network_id", network_id.into()),
            ])),
        )
        .await?)
}

pub async fn get_one(
    State(state): State<RepositoryType>,
    Query((ip, network_id)): Query<(IpAddr, Uuid)>,
) -> Result<impl IntoResponse, ResponseError> {
    let state = state.lock().await;

    let device = state
        .get::<Device>(Some(HashMap::from([
            ("ip", TypeTable::String(ip.to_string())),
            ("network_id", TypeTable::String(network_id.to_string())),
        ])))
        .await?;

    Ok(Json(json!({
        "device": device.first()
    })))
}

pub async fn delete(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Query((ip, network_id)): Query<(IpAddr, Uuid)>,
) -> Result<impl IntoResponse, ResponseError> {
    if role != Role::Admin {
        return Err(ResponseError::Unauthorized);
    }

    let state = state.lock().await;

    Ok(state
        .delete::<Device>(Some(HashMap::from([
            ("ip", ip.into()),
            ("network_id", network_id.into()),
        ])))
        .await?)
}

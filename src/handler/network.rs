use super::*;

use super::RepositoryType;
use crate::models::{device::Device, network::*};

pub async fn create(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Json(netw): Json<models_data_entry::Network>,
) -> Result<impl IntoResponse, ResponseError> {
    unimplemented!()
}

pub async fn get_one(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    todo!()
}

pub async fn update(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Path(id): Path<Uuid>,
    Json(network): Json<UpdateNetwork>,
) -> Result<impl IntoResponse, ResponseError> {
    todo!()
}

pub async fn get_all(
    State(state): State<RepositoryType>,
) -> Result<impl IntoResponse, ResponseError> {
    todo!()
}

pub async fn delete(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ResponseError> {
    todo!()
}

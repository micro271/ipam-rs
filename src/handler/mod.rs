pub mod device;
pub mod error;
mod models_data_entry;
pub mod network;

use crate::database::{utils::Repository, PgRepository};
use crate::models::{utils::TypeTable, *};
use crate::user::Role;
use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use error::ResponseError;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

type RepositoryType = Arc<Mutex<PgRepository>>;

pub mod auth {
    use super::*;
    use crate::user::{self, create_token, encrypt, verify_pass, Verify};
    use axum::{extract::Request, middleware::Next, response::Response};

    pub async fn create(
        State(state): State<RepositoryType>,
        Extension(role): Extension<Role>,
        Json(mut user): Json<user::User>,
    ) -> Result<impl IntoResponse, ResponseError> {
        if role != Role::Admin {
            return Err(ResponseError::Unauthorized);
        }

        let state = state.lock().await;

        user.password = match encrypt(user.password.as_ref()) {
            Ok(e) => e,
            Err(_) => return Err(ResponseError::ServerError),
        };

        Ok(state.insert(vec![user]).await?)
    }

    pub async fn login(
        State(state): State<RepositoryType>,
        Json(user): Json<models_data_entry::User>,
    ) -> Result<impl IntoResponse, ResponseError> {
        let state = state.lock().await;

        let resp = state
            .get::<'_, user::User>(Some(HashMap::from([("username", user.username.into())])))
            .await?.remove(0);
        
        match verify_pass(user.password.as_ref(), &resp.password) {
            Verify::Ok(true) => match create_token(&resp) {
                Ok(e) => Ok(Json(json!({"token":e}))),
                Err(_) => Err(ResponseError::ServerError),
            },
            _ => Err(ResponseError::Unauthorized),
        }
    }

    pub async fn verify_token(mut req: Request, next: Next) -> Result<Response, ResponseError> {
        match req.headers().get(axum::http::header::AUTHORIZATION) {
            Some(e) => match e.to_str() {
                Ok(e) => match e.split(' ').collect::<Vec<_>>().get(1) {
                    Some(e) => match user::verify_token(e) {
                        Ok(Verify::Ok(e)) => {
                            req.extensions_mut().insert(e.role);
                            Ok(next.run(req).await)
                        }
                        _ => Err(ResponseError::Unauthorized),
                    },
                    None => Err(ResponseError::StatusCode(StatusCode::BAD_REQUEST)),
                },
                Err(_) => Err(ResponseError::StatusCode(StatusCode::BAD_REQUEST)),
            },
            None => Err(ResponseError::Unauthorized),
        }
    }
}

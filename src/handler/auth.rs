use super::*;
use crate::{models::user::User, services::Claims};
use ipam_backend::{authentication::{encrypt, create_token, verify_passwd, Verify, self}, cookie::Cookie::TOKEN};
use axum::{extract::Request, response::Response, middleware::Next};
use cookie::Cookie;

pub async fn create(
    State(state): State<RepositoryType>,
    Extension(role): Extension<Role>,
    Json(mut user): Json<User>,
) -> Result<impl IntoResponse, ResponseError> {
    if role != Role::Admin {
        return Err(ResponseError::Unauthorized);
    }

    let state = state.lock().await;

    user.password = match encrypt(user.password) {
        Ok(e) => e,
        Err(_) => return Err(ResponseError::ServerError),
    };

    Ok(state.insert(vec![user]).await?)
}

#[axum::debug_handler]
pub async fn login(
    State(state): State<RepositoryType>,
    Json(user): Json<models_data_entry::User>,
) -> Result<Response, ResponseError> {
    let state = state.lock().await;

    let resp = state
        .get::<'_, User>(Some(HashMap::from([("username", user.username.into())])))
        .await?
        .remove(0);

    match verify_passwd(user.password, &resp.password) {
        Verify::Ok(true) => match create_token(Claims::from(resp)) {
            Ok(e) => {

                let c = Cookie::build((TOKEN.to_string(),e))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(cookie::SameSite::None);
                

                Ok(Response::builder()
                    .header(axum::http::header::SET_COOKIE, c.to_string())
                    .status(StatusCode::OK)
                    .body(().into())
                    .unwrap_or_default())
                

            },
            Err(_) => Err(ResponseError::ServerError),
        },
        _ => Err(ResponseError::Unauthorized),
    }
}

pub async fn verify_token(mut req: Request, next: Next) -> Result<axum::response::Response, ResponseError> {
    match req.headers().get(axum::http::header::AUTHORIZATION) {
        Some(e) => match e.to_str() {
            Ok(e) => match e.split(' ').collect::<Vec<_>>().get(1) {
                Some(e) => match authentication::verify_token::<Claims,_>(*e) {
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

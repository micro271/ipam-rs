use super::*;
use crate::{models::user::User, services::Claims};
use axum::{extract::Request, middleware::Next, response::Response};
use cookie::Cookie;
use ipam_backend::{
    authentication::{self, create_token, encrypt, verify_passwd, Verify},
    cookie::Cookie::TOKEN,
};

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
                let c = Cookie::build((TOKEN.to_string(), e))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(cookie::SameSite::None);

                Ok(Response::builder()
                    .header(axum::http::header::SET_COOKIE, c.to_string())
                    .status(StatusCode::OK)
                    .body(().into())
                    .unwrap_or_default())
            }
            Err(_) => Err(ResponseError::ServerError),
        },
        _ => Err(ResponseError::Unauthorized),
    }
}

pub async fn verify_token(
    ipam_backend::Token(token): ipam_backend::Token,
    mut req: Request,
    next: Next,
) -> Result<axum::response::Response, ResponseError> {
    match authentication::verify_token::<Claims, _>(token) {
        Ok(Verify::Ok(e)) => {
            req.extensions_mut().insert(e.role);
            Ok(next.run(req).await)
        }
        _ => Err(ResponseError::Unauthorized),
    }
}

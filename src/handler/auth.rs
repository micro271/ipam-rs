use super::*;
use crate::{models::user::User, services::Claims};
use axum::{extract::Request, middleware::Next, response::Response};
use cookie::Cookie;
use libipam::{
    authentication::{self, create_token, encrypt, verify_passwd},
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

pub async fn login(
    State(state): State<RepositoryType>,
    Json(user): Json<models_data_entry::User>,
) -> Result<Response, ResponseError> {
    let state = state.lock().await;

    let resp = state
        .get::<'_, User>(Some(HashMap::from([("username", user.username.into())])))
        .await?
        .remove(0);

    if verify_passwd(user.password, &resp.password) {
        match create_token(Claims::from(resp)) {
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
        }
    } else {
        Err(ResponseError::Unauthorized)
    }
}

pub async fn verify_token(
    libipam::Token(token): libipam::Token,
    mut req: Request,
    next: Next,
) -> Result<axum::response::Response, ResponseError> {
    
    match token.map(authentication::verify_token::<Claims, _>) {
        Ok(Ok(e)) => {
            req.extensions_mut().insert(e.role);
            Ok(next.run(req).await)
        }
        _ => Err(ResponseError::Unauthorized),
    }
}

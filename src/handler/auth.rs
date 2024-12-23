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
    uri: Uri,
    _: IsAdministrator,
    Json(mut user): Json<User>,
) -> Result<impl IntoResponse, ResponseError> {
    user.password = match encrypt(user.password) {
        Ok(e) => e,
        Err(e) => {
            return Err(ResponseError::builder()
                .detail(e.to_string())
                .title("Encrypting error".to_string())
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .instance(uri.to_string())
                .build())
        }
    };

    Ok(state.insert(vec![user]).await?)
}

pub async fn login(
    State(state): State<RepositoryType>,
    uri: Uri,
    Json(user): Json<entries::models::User>,
) -> Result<Response, ResponseError> {
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
            Err(_) => Err(ResponseError::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .build()),
        }
    } else {
        Err(ResponseError::unauthorized(
            &uri,
            Some("invalid username or password".to_string()),
        ))
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
        _ => Err(ResponseError::unauthorized(
            req.uri(),
            Some("invalid username o password".to_string()),
        )),
    }
}

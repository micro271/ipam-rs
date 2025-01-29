use super::*;
use crate::{database::repository::QueryResult, models::user::User, services::Claims};
use axum::{extract::Request, middleware::Next, response::Response};
use cookie::Cookie;
use libipam::{
    authentication::{self, create_token, encrypt, verify_passwd},
    GetToken, TokenCookie, TOKEN_PEER_KEY,
};
use models::user::UpdateUser;

#[instrument(level = Level::DEBUG)]
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

    Ok(state.insert(user).await?)
}
#[instrument(level = Level::INFO)]
pub async fn update(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
    Json(updater): Json<UpdateUser>,
) -> Result<QueryResult<User>, ResponseError> {
    Ok(state
        .update::<User, _>(updater, Some(HashMap::from([("id", id.into())])))
        .await?)
}

#[instrument(level = Level::DEBUG)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
) -> Result<QueryResult<User>, ResponseError> {
    let user = state
        .get::<User>(Some(HashMap::from([("id", id.into())])), None, None)
        .await?
        .remove(0);

    if user.is_admin() {
        let user = state
            .get::<User>(
                Some(HashMap::from([("role", Role::Admin.into())])),
                None,
                None,
            )
            .await
            .unwrap_or_default();
        if user.len() <= 1 {
            return Err(ResponseError::builder()
                .detail("The system requires at least one administrator".to_string())
                .build());
        }
    }

    Ok(state
        .delete(Some(HashMap::from([("id", id.into())])))
        .await?)
}

#[instrument(level = Level::INFO)]
pub async fn login(
    State(state): State<RepositoryType>,
    uri: Uri,
    Json(entries::models::User { username, password }): Json<entries::models::User>,
) -> Result<Response, ResponseError> {
    let resp = state
        .get::<User>(
            Some(HashMap::from([("username", username.clone().into())])),
            None,
            None,
        )
        .await?
        .remove(0);

    if let Some(Ok(e)) =
        verify_passwd(password, &resp.password).then_some(create_token(Claims::from(resp)))
    {
        let last_login = Some(time::OffsetDateTime::now_utc());

        state
            .update::<User, _>(
                UpdateUser {
                    last_login,
                    ..Default::default()
                },
                Some(HashMap::from([("username", username.into())])),
            )
            .await?;

        let c = Cookie::build((TOKEN_PEER_KEY.to_string(), e.clone()))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::None);

        Ok(Response::builder()
            .header(axum::http::header::SET_COOKIE, c.to_string())
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .status(StatusCode::OK)
            .body(
                serde_json::json!({
                    "data": {
                        "token": e,
                    },
                    "status": 200,
                    "success": true,
                })
                .to_string()
                .into(),
            )
            .unwrap_or_default())
    } else {
        Err(ResponseError::unauthorized(
            &uri,
            Some("invalid username or password".to_string()),
        ))
    }
}

#[instrument(level = Level::DEBUG)]
pub async fn verify_token(
    libipam::Token(token): libipam::Token<TokenCookie>,
    mut req: Request,
    next: Next,
) -> Result<axum::response::Response, ResponseError> {
    let claim = authentication::verify_token::<Claims, _>(token.get()).map_err(|_| {
        ResponseError::unauthorized(req.uri(), Some("Username or password invalid".to_string()))
    })?;
    req.extensions_mut().insert(claim.role);
    Ok(next.run(req).await)
}

use super::{
    HashMap, IsAdministrator, Json, Level, Path, Repository, RepositoryType, ResponseError, Role,
    State, StatusCode, Uri, Uuid, entries, entries::models::UserEntry, instrument, models,
};
use crate::{
    models::{network::addresses::Addresses, user::User},
    response::ResponseQuery,
    services::Claims,
};
use axum::{extract::Request, middleware::Next, response::Response};
use cookie::Cookie;
use libipam::{
    GetToken, TOKEN_PEER_KEY, TokenAuth,
    services::authentication::{self, create_token, encrypt, verify_passwd},
};
use models::user::UpdateUser;
use serde_json::{Value, json};

#[instrument(level = Level::DEBUG)]
pub async fn create(
    State(state): State<RepositoryType>,
    uri: Uri,
    _: IsAdministrator,
    Json(mut user): Json<UserEntry>,
) -> Result<ResponseQuery<(), Value>, ResponseError> {
    user.password = tokio::task::spawn_blocking(move || {
        encrypt(user.password).map_err(|_| {
            ResponseError::builder()
                .detail("Encrypt error".to_string())
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .build()
        })
    })
    .await
    .map_err(|_| {
        ResponseError::builder()
            .detail("Thread pool error".to_string())
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .build()
    })??;

    Ok(state.insert(User::from(user)).await?.into())
}
#[instrument(level = Level::INFO)]
pub async fn update(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
    Json(updater): Json<UpdateUser>,
) -> Result<ResponseQuery<Addresses, Value>, ResponseError> {
    let update = state
        .update::<User, _>(updater, Some(HashMap::from([("id", id.into())])))
        .await?;

    Ok(ResponseQuery::new(
        None,
        Some(json!(update)),
        None,
        StatusCode::OK,
    ))
}

#[instrument(level = Level::DEBUG)]
pub async fn delete(
    State(state): State<RepositoryType>,
    Path(id): Path<Uuid>,
) -> Result<ResponseQuery<Addresses, Value>, ResponseError> {
    let user = state
        .get::<User>(Some([("id", id.into())].into()), None, None)
        .await?
        .remove(0);

    if user.is_admin() {
        let user = state
            .get::<User>(Some([("role", Role::Admin.into())].into()), None, None)
            .await?;

        if user.len() <= 1 {
            return Err(ResponseError::builder()
                .detail("The system requires at least one administrator".to_string())
                .build());
        }
    }
    let resp = state
        .delete::<Addresses>(Some([("id", id.into())].into()))
        .await?;

    Ok(ResponseQuery::new(
        None,
        Some(json!(resp)),
        None,
        StatusCode::OK,
    ))
}

#[instrument(level = Level::INFO)]
pub async fn login(
    State(state): State<RepositoryType>,
    uri: Uri,
    Json(entries::models::UserEntry { username, password }): Json<entries::models::UserEntry>,
) -> Result<Response, ResponseError> {
    let resp = state
        .get::<User>(
            Some([("username", username.clone().into())].into()),
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
                Some([("username", username.into())].into()),
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
    libipam::Token(token): libipam::Token<TokenAuth>,
    mut req: Request,
    next: Next,
) -> Result<axum::response::Response, ResponseError> {
    let uri = req.uri().clone();
    let claim = tokio::task::spawn_blocking(move || {
        authentication::verify_token::<Claims, _>(token.get()).map_err(|_| {
            ResponseError::unauthorized(&uri, Some("Username or password invalid".to_string()))
        })
    })
    .await
    .map_err(|_| {
        ResponseError::builder()
            .detail("Thread Pool Error".to_string())
            .instance(req.uri().to_string())
            .title("Login error".to_string())
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .build()
    })??;

    req.extensions_mut().insert(claim.role);
    Ok(next.run(req).await)
}

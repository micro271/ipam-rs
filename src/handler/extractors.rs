use super::{ResponseError, Role};
use axum::{extract::FromRequestParts, http::request::Parts};

pub struct IsAdministrator;

impl<S> FromRequestParts<S> for IsAdministrator
where
    S: Send + Sync,
{
    type Rejection = ResponseError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let resp = async {
            match parts.extensions.get::<Role>() {
                Some(e) if e == &Role::Admin => Ok(Self),
                _ => Err(ResponseError::unauthorized(
                    &parts.uri,
                    Some("This function is only allowed for the Admin role".to_string()),
                )),
            }
        };
        resp.await
    }
}

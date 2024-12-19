use super::{ResponseError, Role};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::{future::Future, pin::Pin};

pub struct IsAdministrator;

impl<S> FromRequestParts<S> for IsAdministrator {
    type Rejection = ResponseError;
    fn from_request_parts<'a, 'b, 'c>(
        parts: &'a mut Parts,
        _state: &'b S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'c>>
    where
        'a: 'c,
        'b: 'c,
        Self: 'c,
    {
        let resp = async {
            match parts.extensions.get::<Role>() {
                Some(e) if e == &Role::Admin => Ok(Self),
                _ => Err(ResponseError::unauthorized(
                    &parts.uri,
                    Some("This function is only allowed for the Admin role".to_string()),
                )),
            }
        };

        Box::pin(resp)
    }
}

use axum::{extract::FromRequestParts, http::StatusCode};
use error::NotFound;
use futures::FutureExt;
use std::convert::Infallible;
use std::{boxed::Box, future::Future, pin::Pin};
pub struct Token(pub Result<String, NotFound>);

pub struct Theme(pub theme::Theme);

impl<S> FromRequestParts<S> for Token
where
    S: Send,
{
    type Rejection = Infallible;
    fn from_request_parts<'a, 'b, 'c>(
        parts: &'a mut axum::http::request::Parts,
        _state: &'b S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'c>>
    where
        'a: 'c,
        'b: 'c,
    {
        async {
            let cookies = parts.headers.get(axum::http::header::COOKIE);
            if let Some(Ok(tmp)) =
                cookies.map(|e| e.to_str().map(|x| x.split(';').collect::<Vec<_>>()))
            {
                for i in tmp {
                    let cookie: Vec<_> = i.split("=").collect();
                    if let (Some(Ok(cookie::Cookie::TOKEN)), Some(value)) = (
                        cookie.first().map(|x| cookie::Cookie::try_from(*x)),
                        cookie.get(1),
                    ) {
                        return Ok(Self(Ok(value.to_string())));
                    }
                }
            }
            Ok(Self(Err(NotFound {
                key: cookie::Cookie::TOKEN.to_string(),
            })))
        }
        .boxed()
    }
}

impl<S> FromRequestParts<S> for Theme
where
    S: Send,
{
    type Rejection = Infallible;

    fn from_request_parts<'a, 'b, 'c>(
        parts: &'a mut axum::http::request::Parts,
        _state: &'b S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'c>>
    where
        'a: 'c,
        'b: 'c,
    {
        async {
            if let Some(e) = parts.headers.get(axum::http::header::COOKIE) {
                if let Ok(key_value) = e.to_str().map(|x| x.split(';').collect::<Vec<_>>()) {
                    for i in key_value {
                        let tmp: Vec<_> = i.split('=').collect();
                        if let (Some(Ok(self::cookie::Cookie::THEME)), Some(value)) = (
                            tmp.first().map(|x| self::cookie::Cookie::try_from(*x)),
                            tmp.get(1),
                        ) {
                            return Ok(Self(match self::theme::Theme::try_from(*value) {
                                Ok(e) => e,
                                _ => theme::Theme::Light,
                            }));
                        }
                    }
                }
            }
            Ok(Theme(theme::Theme::Light))
        }
        .boxed()
    }
}

pub mod cookie {
    #[derive(Debug, PartialEq)]
    pub enum Cookie {
        TOKEN,
        THEME,
    }

    impl std::fmt::Display for Cookie {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::TOKEN => write!(f, "jwt"),
                Self::THEME => write!(f, "theme"),
            }
        }
    }

    impl TryFrom<&str> for Cookie {
        type Error = super::error::ParseError;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "jwt" => Ok(Self::TOKEN),
                "theme" => Ok(Self::THEME),
                _ => Err(super::error::ParseError),
            }
        }
    }
}

pub mod theme {

    #[derive(Debug, PartialEq)]
    pub enum Theme {
        Dark,
        Light,
    }

    impl std::fmt::Display for Theme {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use Theme::*;

            match self {
                Dark => write!(f, "dark"),
                Light => write!(f, "light"),
            }
        }
    }

    impl TryFrom<&str> for Theme {
        type Error = super::error::ParseError;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "dark" => Ok(Self::Dark),
                "light" => Ok(Self::Light),
                _ => Err(super::error::ParseError),
            }
        }
    }
}

pub mod error {
    use super::StatusCode;
    use axum::response::IntoResponse;

    #[derive(Debug)]
    pub struct NotFound {
        pub key: String,
    }

    impl IntoResponse for NotFound {
        fn into_response(self) -> axum::response::Response {
            (StatusCode::NOT_FOUND, format!("{} not found", self.key)).into_response()
        }
    }

    #[derive(Debug)]
    pub struct ParseError;
}

pub mod authentication {
    use bcrypt::{hash, verify, DEFAULT_COST};
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use serde::{de::DeserializeOwned, Serialize};
    use std::sync::LazyLock;

    static ALGORITHM_JWT: LazyLock<Algorithm> = LazyLock::new(|| Algorithm::HS256);

    pub trait Claim: std::fmt::Debug {}

    pub fn verify_passwd<T: AsRef<[u8]>>(pass: T, pass_db: &str) -> bool {
        verify(pass.as_ref(), pass_db).unwrap_or(false)
    }

    pub fn encrypt<T: AsRef<[u8]>>(pass: T) -> Result<String, error::Error> {
        Ok(hash(pass.as_ref(), DEFAULT_COST)?)
    }

    pub fn create_token<T>(claim: T) -> Result<String, error::Error>
    where
        T: Serialize + Claim,
    {
        let secret = std::env::var("SECRET_KEY")?;

        Ok(encode(
            &Header::new(*ALGORITHM_JWT),
            &claim,
            &EncodingKey::from_secret(secret.as_ref()),
        )?)
    }

    pub fn verify_token<T, B: AsRef<str>>(token: B) -> Result<T, error::Error>
    where
        T: DeserializeOwned + Claim,
    {
        let secret = std::env::var("SECRET_KEY")?;

        match decode(
            token.as_ref(),
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::new(*ALGORITHM_JWT),
        ) {
            Ok(e) => Ok(e.claims),
            Err(e) => Err(e.into()),
        }
    }

    pub mod error {
        #[derive(Debug)]
        pub enum Error {
            Encrypt,
            EncodeToken,
            SecretKey,
        }

        impl std::fmt::Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Error::Encrypt => write!(f, "Encrypt Error"),
                    Error::EncodeToken => write!(f, "Encode Token Error"),
                    Error::SecretKey => write!(f, "Secret key not found"),
                }
            }
        }

        impl std::error::Error for Error {}

        impl From<std::env::VarError> for Error {
            fn from(_value: std::env::VarError) -> Self {
                Self::SecretKey
            }
        }

        impl From<jsonwebtoken::errors::Error> for Error {
            fn from(_value: jsonwebtoken::errors::Error) -> Self {
                Self::EncodeToken
            }
        }

        impl From<bcrypt::BcryptError> for Error {
            fn from(_value: bcrypt::BcryptError) -> Self {
                Self::Encrypt
            }
        }
    }
}

#[allow(dead_code)]
mod response_error {
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
    };
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ResponseError {
        #[serde(skip_serializing_if = "Option::is_none")]
        r#type: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<u16>,

        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        instance: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<OffsetDateTime>,
    }

    impl ResponseError {
        pub fn new(
            r#type: String,
            title: String,
            status: StatusCode,
            detail: String,
            instance: String,
        ) -> Self {
            Self {
                r#type: Some(r#type),
                title: Some(title),
                status: Some(status.as_u16()),
                detail: Some(detail),
                instance: Some(instance),
                timestamp: Some(OffsetDateTime::now_utc()),
            }
        }

        pub(self) fn create(
            Builder {
                r#type,
                title,
                status,
                detail,
                instance,
            }: Builder,
        ) -> ResponseError {
            Self {
                r#type,
                title,
                status,
                detail,
                instance,
                timestamp: Some(OffsetDateTime::now_utc()),
            }
        }
    }

    impl IntoResponse for ResponseError {
        fn into_response(self) -> axum::response::Response {
            Response::builder()
                .header(axum::http::header::CONTENT_TYPE, "application/problem+json")
                .status(StatusCode::from_u16(self.status.unwrap()).unwrap())
                .body(serde_json::json!(self).to_string())
                .unwrap_or_default()
                .into_response()
        }
    }

    pub struct Builder {
        pub r#type: Option<String>,
        pub title: Option<String>,
        pub status: Option<u16>,
        pub detail: Option<String>,
        pub instance: Option<String>,
    }

    impl Builder {
        pub fn new(status: StatusCode) -> Self {
            Self {
                r#type: None,
                title: None,
                status: Some(status.as_u16()),
                detail: None,
                instance: None,
            }
        }
        pub fn r#type(mut self, r#type: String) -> Self {
            self.r#type = Some(r#type);
            self
        }

        pub fn title(mut self, title: String) -> Self {
            self.title = Some(title);
            self
        }

        pub fn detail(mut self, detail: String) -> Self {
            self.detail = Some(detail);
            self
        }

        pub fn instance(mut self, instance: String) -> Self {
            self.instance = Some(instance);
            self
        }

        pub fn build(self) -> ResponseError {
            ResponseError::create(self)
        }
    }
}

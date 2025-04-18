pub mod response_error;
pub mod services;
pub mod types;

pub struct Token<T: GetToken>(pub T);

impl<T: GetToken> Token<T> {
    pub fn get_token(self) -> String {
        self.0.get()
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug)]
pub struct TokenCookie(String);

#[derive(Debug)]
pub struct TokenAuth(String);

pub trait GetToken: Send + Sync {
    fn find(value: &axum::http::HeaderMap) -> Option<Self>
    where
        Self: Sized;
    fn get(self) -> String;
}

pub const TOKEN_PEER_KEY: &str = "jwt";

impl GetToken for TokenCookie {
    fn find(value: &axum::http::HeaderMap) -> Option<Self> {
        value
            .iter()
            .find(|(key, _)| key.eq(&axum::http::header::COOKIE))
            .and_then(|(_, value)| value.to_str().ok())
            .and_then(|x| {
                x.split(';')
                    .map(str::trim)
                    .find(|x| x.starts_with(TOKEN_PEER_KEY))
                    .and_then(|x| x.split('=').nth(1).map(|x| Self(x.to_string())))
            })
    }
    fn get(self) -> String {
        self.0
    }
}

impl GetToken for TokenAuth {
    fn find(value: &axum::http::HeaderMap) -> Option<Self> {
        value
            .iter()
            .find(|(x, _)| x.eq(&axum::http::header::AUTHORIZATION))
            .and_then(|(_, value)| {
                value
                    .to_str()
                    .ok()
                    .map(str::trim)
                    .and_then(|x| x.split_whitespace().nth(1).map(|x| Self(x.to_string())))
            })
    }
    fn get(self) -> String {
        self.0
    }
}

impl<S, T> axum::extract::FromRequestParts<S> for Token<T>
where
    S: Send + Sync,
    T: GetToken,
{
    type Rejection = response_error::ResponseError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        T::find(&parts.headers).map(Token).ok_or(
            crate::response_error::ResponseError::unauthorized(
                Some(parts.uri.to_string()),
                Some("Token doesn't present".to_string()),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::*;

    #[test]
    fn search_cookie_token_some() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer 12345").unwrap(),
        );
        headers.insert(
            ORIGIN,
            HeaderValue::from_str("http://localhost.local").unwrap(),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str("jwt=123123123123;test=123123123;tr=lnsdlkansdl").unwrap(),
        );

        let token = TokenCookie::find(&headers);
        assert!(token.is_some());
        assert_eq!(token.unwrap().get(), "123123123123".to_string());
    }

    #[test]
    fn search_cookie_token_some_not_eq() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer 12345").unwrap(),
        );
        headers.insert(
            ORIGIN,
            HeaderValue::from_str("http://localhost.local").unwrap(),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str("jwt=123123123123;test=123123123;tr=lnsdlkansdl").unwrap(),
        );

        let token = TokenCookie::find(&headers);

        assert_ne!(token.unwrap().get(), "123".to_string());
    }

    #[test]
    fn search_cookie_token_none() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer 12345").unwrap(),
        );
        headers.insert(
            ORIGIN,
            HeaderValue::from_str("http://localhost.local").unwrap(),
        );
        headers.insert(CONTENT_LENGTH, HeaderValue::from_str("125").unwrap());

        let token = TokenCookie::find(&headers);
        assert!(token.is_none());
    }

    #[test]
    fn search_cookie_authorization_some() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_str("125").unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer 12345").unwrap(),
        );
        headers.insert(
            ORIGIN,
            HeaderValue::from_str("http://localhost.local").unwrap(),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str("jwt=123123123123;test=123123123;tr=lnsdlkansdl").unwrap(),
        );

        let token = TokenAuth::find(&headers);

        assert!(token.is_some());
        assert_eq!(token.unwrap().get(), "12345".to_string());
    }

    #[test]
    fn search_cookie_authorization_not_eq() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_str("125").unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer 12345").unwrap(),
        );
        headers.insert(
            ORIGIN,
            HeaderValue::from_str("http://localhost.local").unwrap(),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str("jwt=123123123123;test=123123123;tr=lnsdlkansdl").unwrap(),
        );

        let token = TokenAuth::find(&headers);

        assert_ne!(token.unwrap().get(), "fff".to_string());
    }

    #[test]
    fn search_cookie_authorization_none() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_LENGTH, HeaderValue::from_str("125").unwrap());
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            ORIGIN,
            HeaderValue::from_str("http://localhost.local").unwrap(),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str("jwt=123123123123;test=123123123;tr=lnsdlkansdl").unwrap(),
        );

        let token = TokenAuth::find(&headers);

        assert!(token.is_none());
    }
}

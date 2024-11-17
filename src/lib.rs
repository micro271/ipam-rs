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
    use time::{OffsetDateTime, UtcOffset};

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
            offset: Option<UtcOffset>,
        ) -> Self {
            Self {
                r#type: Some(r#type),
                title: Some(title),
                status: Some(status.as_u16()),
                detail: Some(detail),
                instance: Some(instance),
                timestamp: Some(
                    OffsetDateTime::now_utc().to_offset(offset.unwrap_or(UtcOffset::UTC)),
                ),
            }
        }

        pub(self) fn create(
            Builder {
                r#type,
                title,
                status,
                detail,
                instance,
                offset,
            }: Builder,
        ) -> ResponseError {
            Self {
                r#type,
                title,
                status,
                detail,
                instance,
                timestamp: Some(
                    OffsetDateTime::now_utc().to_offset(offset.unwrap_or(UtcOffset::UTC)),
                ),
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
        pub offset: Option<UtcOffset>,
    }

    impl Builder {
        pub fn new(status: StatusCode) -> Self {
            Self {
                r#type: None,
                title: None,
                status: Some(status.as_u16()),
                detail: None,
                instance: None,
                offset: None,
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

        pub fn offset(mut self, offset: time::UtcOffset) -> Self {
            self.offset = Some(offset);
            self
        }

        pub fn offset_hms(mut self, (hours, minutes, seconds): (i8, i8, i8)) -> Self {
            self.offset = UtcOffset::from_hms(hours, minutes, seconds).ok();
            self
        }

        pub fn build(self) -> ResponseError {
            ResponseError::create(self)
        }
    }
}

#[allow(dead_code)]
pub mod type_net {

    pub mod host_count {
        use ipnet::IpNet;
        use serde::{Deserialize, Serialize};

        #[derive(Deserialize, Serialize, Debug)]
        #[serde(transparent)]
        pub struct HostCount(u32);

        #[derive(Debug, PartialEq)]
        pub enum Type {
            Limited,
            Unlimited,
        }

        impl std::fmt::Display for Type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Limited => write!(f, "Limited"),
                    Self::Unlimited => write!(f, "Unlimited"),
                }
            }
        }

        pub struct Prefix(u8);
        #[derive(Debug)]
        pub struct InvalidPrefix;

        impl PartialEq for Prefix {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl<T> PartialEq<T> for Prefix
        where
            T: Into<u8> + Copy,
        {
            fn eq(&self, other: &T) -> bool {
                self.0 == T::into(*other)
            }
        }

        impl PartialOrd for Prefix {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<T> PartialOrd<T> for Prefix
        where
            T: Into<u8> + Copy,
        {
            fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
                Some(self.cmp(&T::into(*other)))
            }
        }

        impl From<&IpNet> for Prefix {
            fn from(value: &IpNet) -> Self {
                Self(value.max_prefix_len() - value.prefix_len())
            }
        }

        impl TryFrom<u8> for Prefix {
            type Error = InvalidPrefix;
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                if value > 128 {
                    Err(InvalidPrefix)
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl std::ops::Deref for Prefix {
            type Target = u8;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl HostCount {
            pub const MAX: u32 = u32::MAX;

            pub fn new(prefix: Prefix) -> Self {
                if prefix > 32 {
                    Self(Self::MAX)
                } else {
                    Self(2u32.pow(*prefix as u32) - 2)
                }
            }

            pub fn unlimited(&self) -> bool {
                self.0 == Self::MAX
            }

            pub fn type_limit(&self, prefix: Prefix) -> Type {
                if prefix > 32 {
                    Type::Unlimited
                } else {
                    Type::Limited
                }
            }

            pub fn add<T: TryInto<u32>>(&mut self, rhs: T) -> Result<(), CountOfRange> {
                self.0 = self.0.checked_add(T::try_into(rhs).map_err(|_| CountOfRange)?).ok_or(CountOfRange)?;
                Ok(())
            }

            pub fn sub<T: TryInto<u32>>(&mut self, rhs: T) -> Result<(), CountOfRange> {
                self.0 = self.0.checked_sub(T::try_into(rhs).map_err(|_|CountOfRange)?).ok_or(CountOfRange)?;
                Ok(())
            }
        }

        impl std::ops::Deref for HostCount {
            type Target = u32;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[derive(Debug)]
        pub struct CountOfRange;
    }

    pub mod vlan {
        use serde::{de::Visitor, Deserialize, Serialize};

        pub struct Vlan(u16);

        impl Vlan {
            pub const MAX: u16 = 4095;

            pub fn vlan_id(id: u16) -> Result<Self, OutOfRange> {
                Ok(Vlan(Self::vlidate(id)?))
            }

            pub fn set_vlan(&mut self, id: u16) -> Result<(), OutOfRange> {
                self.0 = Self::vlidate(id)?;
                Ok(())
            }

            fn vlidate(id: u16) -> Result<u16, OutOfRange> {
                if id > Self::MAX {
                    Err(OutOfRange)
                } else {
                    Ok(id)
                }
            }
        }

        impl std::ops::Deref for Vlan {
            type Target = u16;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[derive(Debug)]
        pub struct OutOfRange;

        impl std::fmt::Display for OutOfRange {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Out of range")
            }
        }
        impl std::error::Error for OutOfRange {}

        impl Serialize for Vlan {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_u16(**self)
            }
        }

        struct VlanVisitor;

        impl<'de> Deserialize<'de> for Vlan {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_any(VlanVisitor)
            }
        }

        impl<'de> Visitor<'de> for VlanVisitor {
            type Value = Vlan;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Vlan id expected")
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Self::visit_str(self, &v)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.parse::<u16>().map(Vlan::vlan_id) {
                    Ok(Ok(e)) => Ok(e),
                    _ => Err(E::custom(OutOfRange.to_string())),
                }
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Vlan::vlan_id(v as u16).map_err(|_| E::custom(OutOfRange.to_string()))
            }
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match Vlan::vlan_id(v) {
                    Ok(e) => Ok(e),
                    _ => Err(E::custom(OutOfRange.to_string())),
                }
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v > Vlan::MAX as u32 {
                    Err(E::custom(OutOfRange.to_string()))
                } else {
                    Ok(Vlan::vlan_id(v as u16).unwrap())
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v > Vlan::MAX as u64 {
                    Err(E::custom(OutOfRange.to_string()))
                } else {
                    Ok(Vlan::vlan_id(v as u16).unwrap())
                }
            }

            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v > Vlan::MAX as u128 {
                    Err(E::custom(OutOfRange.to_string()))
                } else {
                    Ok(Vlan::vlan_id(v as u16).unwrap())
                }
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v < 0 {
                    Err(E::custom(OutOfRange.to_string()))
                } else {
                    Ok(Vlan::vlan_id(v as u16).unwrap())
                }
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v < 0 {
                    Err(E::custom(OutOfRange.to_string()))
                } else {
                    Ok(Vlan::vlan_id(v as u16).unwrap())
                }
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v < 0 || v > Vlan::MAX as i32 {
                    Err(E::custom(OutOfRange.to_string()))
                } else {
                    Ok(Vlan::vlan_id(v as u16).unwrap())
                }
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v < 0 || v > Vlan::MAX as i64 {
                    Err(E::custom(OutOfRange.to_string()))
                } else {
                    Ok(Vlan::vlan_id(v as u16).unwrap())
                }
            }

            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v < 0 || v > Vlan::MAX as i128 {
                    Err(E::custom(OutOfRange.to_string()))
                } else {
                    Ok(Vlan::vlan_id(v as u16).unwrap())
                }
            }
        }
    }
}

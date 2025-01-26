#[cfg(feature = "axum")]
use axum::{extract::FromRequestParts, http::StatusCode};


#[cfg(feature = "axum")]
use std::convert::Infallible;

#[cfg(feature = "axum")]
pub struct Token<T: GetToken>(pub T);

impl<T: GetToken> Token<T> {
    pub fn get_token(self) -> String {
        self.0.get()
    }
}

#[cfg(feature = "axum")]
#[derive(Debug)]
pub struct TokenCookie(String);

#[cfg(feature = "axum")]
#[derive(Debug)]
pub struct TokenAuth(String);

#[cfg(feature = "axum")]
pub trait GetToken 
    where 
        Self:Sized,
{
    fn find(value: &axum::http::HeaderMap) -> Option<Self>;
    fn get(self) -> String;
}

#[cfg(feature = "axum")]
const TOKEN: &str = "jwt";

#[cfg(feature = "axum")]
impl GetToken for TokenCookie {
    fn find(value: &axum::http::HeaderMap) -> Option<Self> {
        value.iter().find(|(key,_)| key.eq(&axum::http::header::COOKIE)).map(|(_, value)| value.to_str().ok()).flatten().map(|x| x.split(";").map(str::trim).find(|x| x.starts_with(TOKEN)).map(|x| x.split("=").nth(1).map(|x| Self(x.to_string()))).flatten()).flatten()
    }
    fn get(self) -> String {
        self.0
    }
}

#[cfg(feature = "axum")]
impl GetToken for TokenAuth {
    fn find(value: &axum::http::HeaderMap) -> Option<Self> {
        value.iter().find(|(x,_)| x.eq(&axum::http::header::AUTHORIZATION)).map(|(_, value)| value.to_str().ok().map(str::trim).map(|x|x.split_whitespace().nth(1).map(|x| Self(x.to_string()))).flatten()).flatten()
    }
    fn get(self) -> String {
        self.0
    }
}

#[cfg(feature = "axum")]
pub struct Theme(pub theme::Theme);

#[cfg(feature = "axum")]
impl<S, T> FromRequestParts<S> for Token<T> 
    where 
        S: Send + Sync,
        T: GetToken + Sync + Send,
{
    type Rejection = crate::response_error::ResponseError;

    async fn from_request_parts (
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {

        match T::find(&parts.headers) {
            Some(e) => {
                Ok(Token(e))
            },
            _ => Err(crate::response_error::ResponseError::unauthorized(&parts.uri, None))
        }
    }
}

#[cfg(feature = "axum")]
impl<S> FromRequestParts<S> for Theme
    where
        S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {

        if let Some(Ok(key_value)) = parts.headers.get(axum::http::header::COOKIE).map(|x| x.to_str().map(|x| x.split(";").collect::<Vec<_>>())) {
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
        Ok(Theme(theme::Theme::Light))
        
    }
}

#[cfg(feature = "cookie")]
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

#[cfg(feature = "cookie")]
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

#[cfg(feature = "axum")]
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

#[cfg(feature = "auth")]
pub mod authentication {
    use bcrypt::{hash, verify, DEFAULT_COST};
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use serde::{de::DeserializeOwned, Serialize};

    const ALGORITHM_JWT: Algorithm = Algorithm::HS256;

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
            &Header::new(ALGORITHM_JWT),
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
            &Validation::new(ALGORITHM_JWT),
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

#[cfg(feature = "axum")]
#[allow(dead_code)]
pub mod response_error {
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
        #[serde(with = "time::serde::rfc3339::option")]
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
                    OffsetDateTime::now_utc()
                        .to_offset(offset.unwrap_or(UtcOffset::from_hms(-3, 0, 0).unwrap())),
                ),
            }
        }

        pub fn builder() -> Builder {
            Builder::default()
        }

        pub fn unauthorized(uri: &axum::http::Uri, detail: Option<String>) -> Self {
            Self {
                r#type: None,
                title: Some(StatusCode::UNAUTHORIZED.to_string()),
                status: Some(StatusCode::UNAUTHORIZED.as_u16()),
                detail,
                instance: Some(uri.to_string()),
                timestamp: Some(
                    time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::from_hms(-3, 0, 0).unwrap()),
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
                status: status.or(Some(400)),
                detail,
                instance,
                timestamp: Some(
                    OffsetDateTime::now_utc().to_offset(offset.unwrap_or(UtcOffset::UTC)),
                ),
            }
        }
    }

    impl From<Builder> for ResponseError {
        fn from(value: Builder) -> Self {
            ResponseError::create(value)
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

    #[derive(Debug, Default)]
    pub struct Builder {
        r#type: Option<String>,
        title: Option<String>,
        status: Option<u16>,
        detail: Option<String>,
        instance: Option<String>,
        offset: Option<UtcOffset>,
    }

    impl Builder {
        pub fn r#type(mut self, r#type: String) -> Self {
            self.r#type = Some(r#type);
            self
        }

        pub fn status(mut self, status: StatusCode) -> Self {
            self.status = Some(status.as_u16());
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

    impl From<ResponseError> for Builder {
        fn from(value: ResponseError) -> Self {
            let ResponseError {
                r#type,
                title,
                status,
                detail,
                instance,
                timestamp,
            } = value;
            Builder {
                r#type,
                title,
                status,
                detail,
                instance,
                offset: timestamp.map(|x| x.offset()),
            }
        }
    }
}

#[allow(dead_code)]
#[cfg(feature = "types")]
pub mod type_net {

    pub mod port {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, Deserialize, Serialize)]
        #[cfg_attr(feature = "sqlx_type", derive(sqlx::Type))]
        pub struct Port(u16);

        impl std::ops::Deref for Port {
            type Target = u16;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for Port {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl Port {
            pub fn new(port: u16) -> Self {
                Port(port)
            }
        }

        impl std::cmp::PartialEq for Port {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl std::cmp::PartialEq<u16> for Port {
            fn eq(&self, other: &u16) -> bool {
                self.0 == *other
            }
        }

        impl std::cmp::PartialEq<Port> for u16 {
            fn eq(&self, other: &Port) -> bool {
                *self == other.0
            }
        }

        #[cfg(test)]
        mod test {
            use super::Port;

            #[test]
            fn eq_port_left_side() {
                let port = Port::new(10);
                assert!(10 == port);
            }

            #[test]
            fn eq_port_right_side() {
                let port = Port::new(10);
                assert!(port == 10);
            }
        }
    }

    pub mod host_count {
        use ipnet::IpNet;
        use serde::{Deserialize, Serialize};

        pub struct Prefix {
            host_part: u8,
            network_part: u8,
        }

        impl Prefix {
            const MAX: u8 = 128;

            pub fn part_host(&self) -> u8 {
                self.host_part
            }
            pub fn set(&mut self, network: &IpNet) {
                *self = Prefix::from(network);
            }
            pub fn set_from_prefix(&mut self, prefix: &Prefix) {
                *self = Self { ..*prefix };
            }
        }
        #[derive(Debug)]
        pub struct InvalidPrefix;

        impl PartialEq for Prefix {
            fn eq(&self, other: &Self) -> bool {
                self.network_part == other.network_part
            }
        }

        impl PartialEq<u8> for Prefix {
            fn eq(&self, other: &u8) -> bool {
                *other == self.network_part
            }
        }

        impl PartialOrd for Prefix {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialOrd<u8> for Prefix {
            fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl From<&IpNet> for Prefix {
            fn from(value: &IpNet) -> Self {
                Self {
                    host_part: value.max_prefix_len() - value.prefix_len(),
                    network_part: value.prefix_len(),
                }
            }
        }

        impl std::ops::Deref for Prefix {
            type Target = u8;
            fn deref(&self) -> &Self::Target {
                &self.network_part
            }
        }

        #[derive(Deserialize, Serialize, Debug, Clone)]
        #[cfg_attr(feature = "sqlx_type", derive(sqlx::Type))]
        #[cfg_attr(feature = "sqlx_type", sqlx(transparent))]
        pub struct HostCount(i32);

        impl HostCount {
            pub const MAX: i32 = 0x00FFFFFF;

            pub fn new(prefix: Prefix) -> Self {
                if prefix.part_host() >= 24 {
                    Self(Self::MAX)
                } else {
                    Self(2i32.pow(prefix.part_host().into()) - 2)
                }
            }

            pub fn add(&mut self, value: u32) -> Result<(), CountOfRange> {
                let tmp = self.0.checked_add_unsigned(value);
                match tmp {
                    Some(e) if e <= Self::MAX => {
                        self.0 = e;
                        Ok(())
                    }
                    _ => {
                        self.0 = Self::MAX;
                        Err(CountOfRange)
                    }
                }
            }

            pub fn sub(&mut self, value: u32) -> Result<(), CountOfRange> {
                let tmp = self.0.checked_sub_unsigned(value);
                match tmp {
                    Some(e) => {
                        self.0 = e;
                        Ok(())
                    }
                    None => {
                        self.0 = 0;
                        Err(CountOfRange)
                    }
                }
            }
        }

        impl From<u32> for HostCount {
            fn from(value: u32) -> Self {
                Self(value as i32)
            }
        }

        impl TryFrom<i32> for HostCount {
            type Error = CountOfRange;
            fn try_from(value: i32) -> Result<Self, Self::Error> {
                if !(0..=Self::MAX).contains(&value) {
                    Err(CountOfRange)
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl std::ops::Deref for HostCount {
            type Target = i32;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::cmp::PartialEq for HostCount {
            fn eq(&self, other: &Self) -> bool {
                self.0.eq(&other.0)
            }
        }

        #[derive(Debug)]
        pub struct CountOfRange;

        #[cfg(test)]
        mod test {
            use crate::type_net::host_count::HostCount;

            use super::Prefix;
            use ipnet::IpNet; //dev-dependencies
            #[test]
            fn test_prefix_instance_exit() {
                let ipnet: IpNet = "172.30.0.30/24".parse().unwrap();
                let pref = Prefix::from(&ipnet);

                let ipnet: IpNet = "172.30.0.30/16".parse().unwrap();
                let pref_2 = Prefix::from(&ipnet);
                assert_eq!(24, *pref);
                assert_eq!(16, *pref_2);
            }

            #[test]
            fn test_prefix_instance_fail() {
                let ipnet: IpNet = "172.30.0.30/25".parse().unwrap();
                let pref = Prefix::from(&ipnet);

                let ipnet: IpNet = "172.30.0.30/13".parse().unwrap();
                let pref_2 = Prefix::from(&ipnet);
                assert_ne!(16, *pref);
                assert_ne!(24, *pref_2);
            }

            #[test]
            fn test_prefix_partial_eq_with_prefix() {
                let ipnet: IpNet = "172.30.0.30/25".parse().unwrap();
                let pref = Prefix::from(&ipnet);

                let ipnet: IpNet = "172.30.0.30/25".parse().unwrap();
                let pref_2 = Prefix::from(&ipnet);
                assert!(pref_2 == pref);
            }

            #[test]
            fn test_prefix_partial_eq_with_integer() {
                let ipnet: IpNet = "172.30.0.30/25".parse().unwrap();
                let pref_2 = Prefix::from(&ipnet);
                assert!(pref_2 == 25);
            }

            #[test]
            fn test_prefix_partial_partial_ord_with_prefix() {
                let ipnet: IpNet = "172.30.0.30/24".parse().unwrap();
                let pref = Prefix::from(&ipnet);

                let ipnet: IpNet = "172.30.0.30/25".parse().unwrap();
                let pref_2 = Prefix::from(&ipnet);
                assert_eq!(pref_2 > pref, true);
                assert_eq!(pref_2 < pref, false);
                assert_ne!(pref_2 < pref, true);
                assert_ne!(pref_2 > pref, false);
                assert!(pref_2 > pref);
            }

            #[test]
            fn test_prefix_partial_partial_ord_with_integer() {
                let ipnet: IpNet = "172.30.0.30/24".parse().unwrap();
                let pref = Prefix::from(&ipnet);
                assert_eq!(pref > 10, true);
                assert_eq!(pref < 10, false);
                assert!(pref > 10);
            }

            #[test]
            fn host_counter_instance_from_prefix() {
                let pref = HostCount::new(Prefix::from(&"172.30.0.0/24".parse::<IpNet>().unwrap()));
                assert_eq!(*pref, 254);
            }
            #[test]
            fn host_counter_instance_from_u32() {
                let pref: HostCount = 10.into();
                assert_eq!(*pref, 10);
                assert_ne!(15, *pref);
            }

            #[test]
            fn host_counter_addition_is_err() {
                let mut pref: HostCount = 10.into();
                assert!(pref.add(HostCount::MAX as u32).is_err());
            }

            #[test]
            fn host_counter_addition_overflow() {
                let mut pref: HostCount = HostCount::MAX.try_into().unwrap();
                assert!(pref.add(20).is_err());
                assert_eq!(HostCount::MAX, *pref);
            }
        }
    }

    pub mod vlan {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize, Clone)]
        #[cfg_attr(feature = "sqlx_type", derive(sqlx::Type))]
        #[cfg_attr(feature = "sqlx_type", sqlx(transparent))]
        pub struct VlanId(i16);

        impl VlanId {
            pub const MAX: i16 = 0x0FFF;

            pub fn new(value: i16) -> Result<Self, OutOfRange> {
                value.try_into()
            }

            pub fn set_vlan(&mut self, id: i16) -> Result<(), OutOfRange> {
                if !(2..=Self::MAX).contains(&id) {
                    Err(OutOfRange)
                } else {
                    self.0 = id;
                    Ok(())
                }
            }
        }

        impl std::cmp::PartialEq for VlanId {
            fn eq(&self, other: &Self) -> bool {
                self.0.eq(&other.0)
            }
        }

        impl std::cmp::PartialEq<i16> for VlanId {
            fn eq(&self, other: &i16) -> bool {
                self.0.eq(other)
            }
        }

        impl TryFrom<i16> for VlanId {
            type Error = OutOfRange;
            fn try_from(value: i16) -> Result<Self, Self::Error> {
                if !(2..=Self::MAX).contains(&value) {
                    Err(OutOfRange)
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl std::default::Default for VlanId {
            fn default() -> Self {
                Self(1)
            }
        }

        impl std::ops::Deref for VlanId {
            type Target = i16;
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

        #[cfg(test)]
        mod test {
            use crate::type_net::vlan::VlanId;

            #[test]
            fn vlan_negative_error() {
                let vlan = VlanId::new(-1);
                assert!(vlan.is_err());
            }

            #[test]
            fn vlan_out_range_error() {
                let vlan = VlanId::new(4096);
                assert!(vlan.is_err());
            }

            #[test]
            fn vlan_ok() {
                let vlan = VlanId::new(4095);
                assert!(vlan.is_ok());
            }

            #[test]
            fn vlan_cmp_with_vlan_eq_false() {
                let one = VlanId::new(4095).unwrap();
                let two = VlanId::new(1094).unwrap();
                assert_eq!(one == two, false);
            }

            #[test]
            fn vlan_cmp_with_vlan_eq_true() {
                let one = VlanId::new(4095).unwrap();
                let two = VlanId::new(4095).unwrap();
                assert!(one == two);
            }

            #[test]
            fn vlan_cmp_with_i16_eq_true() {
                let one = VlanId::new(4095).unwrap();
                assert!(one == 4095);
            }

            #[test]
            fn vlan_cmp_with_i16_eq_false() {
                let one = VlanId::new(4095).unwrap();
                assert_eq!(one == 5, false);
            }

            #[test]
            fn vlan_deref_cmp_with_i16_eq_false() {
                let one = VlanId::new(4095).unwrap();
                assert_eq!(*one == 4, false);
            }

            #[test]
            fn vlan_deref_cmp_with_i16_eq_true() {
                let one = VlanId::new(4095).unwrap();
                assert!(*one == 4095);
            }
        }
    }
}

#[cfg(feature = "ipam_services")]
pub mod ipam_services {
    use std::net::{IpAddr, Ipv4Addr};

    use axum::{
        http::{Response, StatusCode},
        response::IntoResponse,
    };
    use ipnet::IpNet;

    #[derive(Debug)]
    pub struct SubnettingError(String);

    impl std::fmt::Display for SubnettingError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Subnneting error: {}", self.0)
        }
    }

    impl std::error::Error for SubnettingError {}

    pub fn subnetting(ipnet: IpNet, prefix: u8) -> Result<Vec<IpNet>, SubnettingError> {
        let ip = match ipnet.network() {
            IpAddr::V4(ipv4) => ipv4,
            IpAddr::V6(_) => {
                return Err(SubnettingError(
                    "You cannot create an red above an ipv6".to_string(),
                ))
            }
        };

        let mut resp = Vec::new();

        let mut subnet =
            IpNet::new(ip.into(), prefix).map_err(|x| SubnettingError(x.to_string()))?;

        if !ipnet.contains(&subnet) {
            return Err(SubnettingError(format!(
                "The network {} doesnt belong to the network {}",
                subnet, ipnet
            )));
        }

        let sub = 2u32.pow((prefix - ipnet.prefix_len()) as u32);
        let hosts = 2u32.pow((subnet.max_prefix_len() - prefix) as u32);

        resp.push(subnet);

        for i in 1..sub {
            let new = u32::from(ip) + i * hosts;
            subnet = IpNet::new(Ipv4Addr::from(new).into(), prefix).unwrap();
            resp.push(subnet);
        }
        Ok(resp)
    }

    pub async fn ping(ip: IpAddr, timeout_ms: u64) -> Ping {
        let ip = ip.to_string();
        let duration = std::time::Duration::from_millis(timeout_ms)
            .as_secs_f32()
            .to_string();
        let ping = tokio::process::Command::new("ping")
            .args(["-W", &duration, "-c", "1", &ip])
            .output()
            .await;

        match ping {
            Ok(e) if e.status.code().unwrap_or(1) == 0 => Ping::Pong,
            _ => Ping::Fail,
        }
    }

    #[derive(Debug, PartialEq, PartialOrd, serde::Serialize)]
    pub enum Ping {
        Pong,
        Fail,
    }
    impl std::fmt::Display for Ping {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Pong => write!(f, "Pong"),
                Self::Fail => write!(f, "Fail"),
            }
        }
    }
    impl IntoResponse for Ping {
        fn into_response(self) -> axum::response::Response {
            Response::builder()
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .status(StatusCode::OK)
                .body(
                    serde_json::json!({
                        "status": 200,
                        "ping": self.to_string()
                    })
                    .to_string(),
                )
                .unwrap_or_default()
                .into_response()
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use std::sync::LazyLock;
        use tokio::runtime::Runtime;

        static RUNTIME: LazyLock<Runtime> = std::sync::LazyLock::new(|| Runtime::new().unwrap());

        #[test]
        fn sub_net_first_prefix_fifty_six() {
            let ip = "192.168.0.1/24".parse::<IpNet>().unwrap();
            let subnet = subnetting(ip, 26).unwrap();
            let mut ip_result = Vec::new();
            ip_result.push("192.168.0.0/26".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.64/26".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.128/26".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.192/26".parse::<IpNet>().unwrap());
            assert!(subnet.contains(&ip_result[0]));
            assert!(subnet.contains(&ip_result[1]));
            assert!(subnet.contains(&ip_result[2]));
            assert!(subnet.contains(&ip_result[3]));
            assert!(subnet.len() == 4)
        }

        #[test]
        fn sub_net_first_prefix_fifty_eight() {
            let ip = "192.168.0.1/24".parse::<IpNet>().unwrap();
            let subnet = subnetting(ip, 28).unwrap();
            let mut ip_result = Vec::new();
            ip_result.push("192.168.0.0/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.16/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.32/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.48/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.64/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.80/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.96/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.112/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.128/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.144/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.160/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.176/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.192/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.208/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.224/28".parse::<IpNet>().unwrap());
            ip_result.push("192.168.0.240/28".parse::<IpNet>().unwrap());
            assert!(subnet.contains(&ip_result[0]));
            assert!(subnet.contains(&ip_result[1]));
            assert!(subnet.contains(&ip_result[2]));
            assert!(subnet.contains(&ip_result[3]));
            assert!(subnet.contains(&ip_result[4]));
            assert!(subnet.contains(&ip_result[5]));
            assert!(subnet.contains(&ip_result[6]));
            assert!(subnet.contains(&ip_result[7]));
            assert!(subnet.contains(&ip_result[8]));
            assert!(subnet.contains(&ip_result[9]));
            assert!(subnet.contains(&ip_result[10]));
            assert!(subnet.contains(&ip_result[11]));
            assert!(subnet.contains(&ip_result[12]));
            assert!(subnet.contains(&ip_result[13]));
            assert!(subnet.contains(&ip_result[14]));
            assert!(subnet.contains(&ip_result[15]));
            assert!(subnet.len() == 16);
        }

        #[test]
        fn sub_net_first_prefix_fifty_four_above_twenty_one() {
            let ip = "192.168.0.1/21".parse::<IpNet>().unwrap();
            let subnet = subnetting(ip, 24).unwrap();
            assert!(subnet.len() == 8);
        }

        #[test]
        fn sub_net_first_prefix_fifteen_above_twenty_four() {
            let ip = "192.168.0.1/15".parse::<IpNet>().unwrap();
            let subnet = subnetting(ip, 24).unwrap();
            assert!(subnet.len() == 512);
        }

        #[test]
        fn ping_test_pong() {
            let resp = RUNTIME.block_on(async { ping("192.168.0.1".parse().unwrap(), 100).await });
            assert_eq!(Ping::Pong, resp);
        }

        #[test]
        fn ping_test_fail() {
            let resp = RUNTIME.block_on(async { ping("192.168.1.50".parse().unwrap(), 100).await });
            assert_eq!(Ping::Fail, resp);
        }
    }
}
